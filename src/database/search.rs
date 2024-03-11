use std::borrow::BorrowMut;
use std::fmt::format;
use std::num::NonZeroUsize;
use std::sync::Arc;

use anyhow::Error;
use sea_orm::DatabaseConnection;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::Opstamp;
use tantivy::ReloadPolicy;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use tokio::time;
use tokio::time::sleep;

use crate::database::query::get_all_txt;

#[derive(Clone)]
pub struct Fields {
    // 正向索引ID
    pub id: Field,
    // 搜索域
    pub title: Field,
    pub body: Field,
    // 权限控制
    pub level: Field,
}
static mut INDEX: Option<&Index> = None;
static mut WRITER: Option<Arc<RwLock<IndexWriter>>> = None;
static mut READER: Option<&IndexReader> = None;
static mut FIELDS: Option<&Fields> = None;

pub fn get_index() -> &'static Index {
    unsafe { INDEX.expect("NO INDEX!!!") }
}
pub fn get_writer() -> Arc<RwLock<IndexWriter>> {
    unsafe { WRITER.clone().expect("NO WRITER!!!") }
}

pub fn get_reader() -> &'static IndexReader {
    unsafe { READER.expect("NO READER!!!") }
}

pub fn get_fields() -> Fields {
    unsafe { FIELDS.expect("NO FILEDS!!!").clone() }
}

pub fn init_index() {
    println!("Init Index ...");

    let mut schema_builder = Schema::builder();

    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("jieba")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default().set_indexing_options(text_field_indexing);

    schema_builder.add_u64_field("id", INDEXED | STORED);
    schema_builder.add_text_field("title", text_options.clone());
    schema_builder.add_text_field("body", text_options.clone());
    schema_builder.add_u64_field("level", INDEXED | FAST);

    let schema = schema_builder.build();

    let id = schema.get_field("id").unwrap();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    let level = schema.get_field("level").unwrap();
    let fields = Fields {
        id,
        title,
        body,
        level,
    };
    // 建立索引
    let tokenizer = tantivy_jieba::JiebaTokenizer {};
    let index = Index::create_in_ram(schema.clone());
    index.tokenizers().register("jieba", tokenizer);

    let writer = index.writer(1024 * 1024 * 50).unwrap();
    let writer: Arc<RwLock<IndexWriter>> = Arc::new(RwLock::new(writer));
    let reader: tantivy::IndexReader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()
        .unwrap();

    // 设置全局变量

    unsafe {
        let index = Box::new(index);
        INDEX = Some(Box::leak(index));

        let reader = Box::new(reader);
        READER = Some(Box::leak(reader));

        WRITER = Some(writer);

        let fields = Box::new(fields);
        FIELDS = Some(Box::leak(fields));
    }
    println!("Index init finish ...")
}

pub async fn rebuild_search_index(conn: DatabaseConnection) {
    println!("Rebuild Index ...");

    let writer = get_writer();
    let writer_read = writer.read().await;
    let _ = writer_read.delete_all_documents();
    let fields = get_fields();

    let txts = get_all_txt(&conn).await.unwrap();
    let mut id_title_and_join_handlers = Vec::with_capacity(512);

    for txt in txts {
        let id = txt.id;
        let title = txt.title;
        let level = txt.level;
        let read_file = async move {
            let mut f = File::open(format!("data/{}", &txt.hash))
                .await
                .expect(&format!("{} does not exist!!!", txt.hash));
            let mut dst = String::with_capacity(4096);
            let _ = f.read_to_string(&mut dst).await;
            dst
        };
        id_title_and_join_handlers.push((id, title, level, tokio::spawn(read_file)));
    }

    for (id, title, level, jh) in id_title_and_join_handlers {
        let body = jh.await.unwrap();
        let _ = writer_read.add_document(doc!(
            fields.id => id,
            fields.title => title,
            fields.body => body,
            fields.level => level as u64
        ));
    }
    drop(writer_read);
    let mut writer_w = writer.write().await;
    let _ = writer_w.commit();
    println!("Index Rebuilt");
}

fn generate_query_str(field_name: &str, terms: Vec<String>, level: u8) -> String {
    let mut buf = String::new();
    for term in terms {
        buf += &format!("{field_name}:\"{term}\" ");
    }
    let parse_str = format!("({buf}) AND level:[0 TO {level}]");
    parse_str
}

pub async fn commiting() {
    let writer = get_writer();
    loop {
        let opstamp: Opstamp = {
            let mut writer_w = writer.write().await;
            writer_w.commit().unwrap()
        };
        println!("committed with opstamp {opstamp}");
        sleep(time::Duration::from_secs(5)).await;
    }
}

pub enum SearchField {
    Title,
    Body,
}

pub fn search_from_rev_index(
    field: SearchField,
    terms: Vec<String>,
    level: u8,
    limit: usize,
) -> anyhow::Result<Vec<u64>> {
    let fields = get_fields();
    let (field_name, field) = match field {
        SearchField::Title => ("title", fields.title),
        SearchField::Body => ("body", fields.body),
    };

    let parse_str = generate_query_str(field_name, terms, level);

    let reader = get_reader();
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(get_index(), vec![field.clone(), fields.level]);
    let query = query_parser.parse_query(&parse_str)?;
    let docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

    let mut res = Vec::new();
    for (_, doc_add) in docs {
        let doc: Document = searcher.doc(doc_add)?;
        let v = doc.get_first(fields.id).unwrap().as_u64().unwrap();
        res.push(v);
    }
    Ok(res)
}
