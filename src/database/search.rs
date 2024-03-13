use std::cmp::max;
use std::sync::Arc;

use sea_orm::DatabaseConnection;

use tantivy::collector::Count;
use tantivy::collector::FilterCollector;
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

#[derive(Clone, Copy)]
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
    unsafe {
        let reader = READER.expect("NO READER!!!");
        let _ = reader.reload();
        reader
    }
}

pub fn get_fields() -> Fields {
    unsafe { FIELDS.expect("NO FILEDS!!!").clone() }
}

pub fn init_index() {
    println!("-->> {:<12} -- start to init", "INIT_INDEX");

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
        .reload_policy(ReloadPolicy::Manual)
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
    println!("-->> {:<12} -- finish", "INIT_INDEX");
}

pub async fn rebuild_search_index(conn: DatabaseConnection) {
    println!("-->> {:<12} -- rebuiding index", "REBUILD_INDEX");

    let writer = get_writer();
    let mut writer_w = writer.write().await;
    let _ = writer_w.delete_all_documents();
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

    let mut count = 0;
    for (id, title, level, jh) in id_title_and_join_handlers {
        let body = jh.await.unwrap();
        let _ = writer_w.add_document(doc!(
            fields.id => id,
            fields.title => title,
            fields.body => body,
            fields.level => level as u64
        ));
        count += 1;
        if count == 10 {
            count = 0;
            let _ = writer_w.commit();
        }
    }
    let _ = writer_w.commit();
    println!("-->> {:<12} -- finish", "REBUILD_INDEX");
}

pub async fn commiting() {
    let writer = get_writer();
    loop {
        let opstamp: Opstamp = {
            let mut writer_w = writer.write().await;
            writer_w.commit().unwrap()
        };
        println!(
            "-->> {:<12} -- committed with opstamp {opstamp:?}",
            "COMMITING"
        );
        sleep(time::Duration::from_secs(5)).await;
    }
}

#[derive(Clone, Copy)]
pub enum SearchField {
    Title,
    Body,
    All,
}

impl From<String> for SearchField {
    fn from(value: String) -> Self {
        if value.eq_ignore_ascii_case("title") {
            SearchField::Title
        } else if value.eq_ignore_ascii_case("body") {
            SearchField::Body
        } else {
            SearchField::All
        }
    }
}

impl From<SearchField> for Vec<Field> {
    fn from(value: SearchField) -> Self {
        let fields = get_fields();
        let search_fields = match value {
            SearchField::Title => vec![fields.title],
            SearchField::Body => vec![fields.body],
            SearchField::All => vec![fields.body, fields.title],
        };
        search_fields
    }
}

pub fn count_match_doc(field: SearchField, query_string: &str, level: u8) -> anyhow::Result<usize> {
    let fields = get_fields();
    let search_fields = Vec::from(field);

    let reader = get_reader();
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(get_index(), search_fields);
    let query = query_parser.parse_query(query_string)?;

    let filter = FilterCollector::new(fields.level, move |v: u64| v <= level as u64, Count);
    let count = searcher.search(&query, &filter)?;
    Ok(count)
}

pub fn search_from_rev_index(
    field: SearchField,
    query_string: &str,
    level: u8,
    limit: usize,
) -> anyhow::Result<Vec<u64>> {
    // 若limit为0，自动设置limit值
    let limit = if limit == 0 {
        max(1, count_match_doc(field, query_string, level)?)
    } else {
        limit
    };

    let fields = get_fields();
    let search_fields = Vec::from(field);

    let reader = get_reader();
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(get_index(), search_fields);
    let query = query_parser.parse_query(query_string)?;

    let filter = FilterCollector::new(
        fields.level,
        move |v: u64| v <= level as u64,
        TopDocs::with_limit(limit),
    );
    let docs = searcher.search(&query, &filter)?;

    let mut res = Vec::new();
    for (_, doc_add) in docs {
        let doc: Document = searcher.doc(doc_add)?;
        let v = doc.get_first(fields.id).unwrap().as_u64().unwrap();
        res.push(v);
    }
    Ok(res)
}

pub async fn delete_from_index(id: u64) -> anyhow::Result<()> {
    let term = Term::from_field_u64(get_fields().id, id);
    let writer = get_writer();
    let writer = writer.read().await;

    let _ = writer.delete_term(term);

    Ok(())
}
