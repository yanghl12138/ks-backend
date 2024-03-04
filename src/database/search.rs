use std::borrow::BorrowMut;

use sea_orm::DatabaseConnection;
use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::ReloadPolicy;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::database::query::get_all_txt;

pub struct Fields {
    pub Id: Field,
    pub Title: Field,
    pub Body: Field
}
static mut INDEX: Option<&Index> = None;
static mut WRITER: Option<&mut IndexWriter> = None;
static mut READER: Option<&IndexReader> = None;
static mut FIELDS: Option<&Fields> = None;

pub fn get_writer() -> &'static mut IndexWriter {
    unsafe {
        WRITER.as_mut().expect("NO WRITER!!!")
    }
}

pub fn get_fields() -> &'static Fields {
    unsafe {
        FIELDS.expect("NO FILEDS!!!")
    }
}

pub fn init_index() {
    println!("Init Index ...");

    let mut schema_builder = Schema::builder();

    let text_field_indexing = TextFieldIndexing::default()
    .set_tokenizer("jieba")
    .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
    .set_indexing_options(text_field_indexing);

    schema_builder.add_u64_field("id", INDEXED | STORED);
    schema_builder.add_text_field("title", text_options.clone());
    schema_builder.add_text_field("body", text_options.clone());

    let schema = schema_builder.build();

    let id = schema.get_field("id").unwrap();
    let title = schema.get_field("title").unwrap();
    let body= schema.get_field("body").unwrap();
    let fields = Fields {
        Id: id,
        Title: title,
        Body: body
    };
    // 建立索引
    let tokenizer = tantivy_jieba::JiebaTokenizer {};
    let index = Index::create_in_ram(schema.clone());
    index.tokenizers().register("jieba", tokenizer);

    let mut writer = index.writer(1024 * 1024 * 50).unwrap();

    let reader: tantivy::IndexReader = index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)
    .try_into().unwrap();

    // 设置全局变量
    unsafe {
        let index = Box::new(index);
        INDEX = Some(Box::leak(index));

        let writer = Box::new(writer);
        WRITER = Some(Box::leak(writer));

        let reader = Box::new(reader);
        READER = Some(Box::leak(reader));

        let fields = Box::new(fields);
        FIELDS = Some(Box::leak(fields));
    }
    println!("Index init finish ...")

}

pub async fn rebuild_search_index(conn: DatabaseConnection) {
    println!("Rebuild Index ...");

    let writer: &mut IndexWriter = get_writer();
    let _ = writer.delete_all_documents();
    let fields = get_fields();
    

    let txts = get_all_txt(&conn).await.unwrap();
    let mut id_title_and_join_handlers = Vec::with_capacity(512);

    for txt in txts {
        let id = txt.id;
        let title = txt.title;
        let read_file = async move {
            let mut f = File::open(
                format!("data/{}", &txt.hash)
            )
            .await.expect(&format!("{} does not exist!!!", txt.hash));
            let mut dst = String::with_capacity(4096);
            let _ = f.read_to_string(&mut dst).await;
            dst
        };
        id_title_and_join_handlers.push((id, title, tokio::spawn(read_file)));
    }

    for (id, title, jh) in id_title_and_join_handlers {
        let body = jh.await.unwrap();
        let _ = writer.add_document(doc!(
            fields.Id => id, 
            fields.Title => title,
            fields.Body => body
        ));
    }
    let _ = writer.commit();
}