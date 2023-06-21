//! Copyright The NoXF/oss-rust-sdk Authors
//! Copyright The iFREEGROUP/oss-rust-sdk Contributors
use quick_xml::{events::Event, Reader};
use std::collections::HashMap;

use crate::oss::RequestType;

use super::errors::{OSSError};
use super::oss::OSS;
use super::utils::*;

#[derive(Clone, Debug)]
pub struct ListObjects {
    bucket_name: String,
    delimiter: String,
    prefix: String,
    marker: String,
    max_keys: String,
    is_truncated: bool,

    objects: Vec<Object>,
}

impl ListObjects {
    pub fn new(
        bucket_name: String,
        delimiter: String,
        prefix: String,
        marker: String,
        max_keys: String,
        is_truncated: bool,

        objects: Vec<Object>,
    ) -> Self {
        ListObjects {
            bucket_name,
            delimiter,
            prefix,
            marker,
            max_keys,
            is_truncated,

            objects,
        }
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn marker(&self) -> &str {
        &self.marker
    }

    pub fn max_keys(&self) -> &str {
        &self.max_keys
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn objects(&self) -> &Vec<Object> {
        &self.objects
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    key: String,
    last_modified: String,
    size: usize,
    etag: String,
    r#type: String,
    storage_class: String,
    owner_id: String,
    owner_display_name: String,
}

impl Object {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: String,
        last_modified: String,
        size: usize,

        etag: String,
        r#type: String,
        storage_class: String,
        owner_id: String,
        owner_display_name: String,
    ) -> Self {
        Object {
            key,
            last_modified,
            size,
            etag,
            r#type,
            storage_class,
            owner_id,
            owner_display_name,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn last_modified(&self) -> &str {
        &self.last_modified
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn storage_class(&self) -> &str {
        &self.storage_class
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn owner_display_name(&self) -> &str {
        &self.owner_display_name
    }
}

pub trait ObjectAPI {
    fn list_object<S, H, R>(&self, headers: H, resources: R) -> Result<ListObjects, OSSError>
    where
        S: AsRef<str>,
        H: Into<Option<HashMap<S, S>>>,
        R: Into<Option<HashMap<S, Option<S>>>>;

    fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Vec<u8>, OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>;

    fn get_object_acl<S>(&self, object_name: S) -> Result<String, OSSError>
    where
        S: AsRef<str>;

    fn put_object_from_file<S1, S2, S3, H, R>(
        &self,
        file: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>;

    fn put_object_from_buffer<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>;

    fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        dest: S2,
        headers: H,
        resources: R,
    ) -> Result<(), OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>;

    fn delete_object<S>(&self, object_name: S) -> Result<(), OSSError>
    where
        S: AsRef<str>;
}

impl<'a> ObjectAPI for OSS<'a> {
    fn list_object<S, H, R>(&self, headers: H, resources: R) -> Result<ListObjects, OSSError>
    where
        S: AsRef<str>,
        H: Into<Option<HashMap<S, S>>>,
        R: Into<Option<HashMap<S, Option<S>>>>,
    {
        let (host, headers) =
            self.build_request(RequestType::Get, String::new(), headers, resources)?;

        let resp = reqwest::blocking::Client::new()
            .get(&host)
            .headers(headers)
            .send()?;

        let xml_str = resp.text()?;
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str.as_str());
        reader.trim_text(true);

        let mut bucket_name = String::new();
        let mut prefix = String::new();
        let mut marker = String::new();
        let mut max_keys = String::new();
        let mut delimiter = String::new();
        let mut is_truncated = false;

        let mut key = String::new();
        let mut last_modified = String::new();
        let mut etag = String::new();
        let mut r#type = String::new();
        let mut size = 0usize;
        let mut storage_class = String::new();
        let mut owner_id = String::new();
        let mut owner_display_name = String::new();

        let list_objects;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"Name" => bucket_name = reader.read_text(e.name())?.to_string(),
                    b"Prefix" => prefix = reader.read_text(e.name())?.to_string(),
                    b"Marker" => marker = reader.read_text(e.name())?.to_string(),
                    b"MaxKeys" => max_keys = reader.read_text(e.name())?.to_string(),
                    b"Delimiter" => delimiter = reader.read_text(e.name())?.to_string(),
                    b"IsTruncated" => {
                        is_truncated = reader.read_text(e.name())? == "true"
                    }
                    b"Contents" => {
                        // do nothing
                    }
                    b"Key" => key = reader.read_text(e.name())?.to_string(),
                    b"LastModified" => last_modified = reader.read_text(e.name())?.to_string(),
                    b"ETag" => etag = reader.read_text(e.name())?.to_string(),
                    b"Type" => r#type = reader.read_text(e.name())?.to_string(),
                    b"Size" => size = reader.read_text(e.name())?.parse::<usize>().unwrap(),
                    b"StorageClass" => storage_class = reader.read_text(e.name())?.to_string(),
                    b"Owner" => {
                        // do nothing
                    }
                    b"ID" => owner_id = reader.read_text(e.name())?.to_string(),
                    b"DisplayName" => owner_display_name = reader.read_text(e.name())?.to_string(),

                    _ => (),
                },

                Ok(Event::End(ref e)) if e.name().as_ref() == b"Contents" => {
                    let object = Object::new(
                        key.clone(),
                        last_modified.clone(),
                        size,
                        etag.clone(),
                        r#type.clone(),
                        storage_class.clone(),
                        owner_id.clone(),
                        owner_display_name.clone(),
                    );
                    result.push(object);
                }
                Ok(Event::Eof) => {
                    list_objects = ListObjects::new(
                        bucket_name,
                        delimiter,
                        prefix,
                        marker,
                        max_keys,
                        is_truncated,
                        result,
                    );
                    break;
                } // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }
        Ok(list_objects)
    }

    fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Vec<u8>, OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>,
    {
        let (host, headers) =
            self.build_request(RequestType::Get, object_name, headers, resources)?;

        let mut resp = reqwest::blocking::Client::new()
            .get(host)
            .headers(headers)
            .send()?;
        let mut buf: Vec<u8> = vec![];

        if resp.status().is_success() {
            resp.copy_to(&mut buf)?;
            Ok(buf)
        } else {
            Err(OSSError::Object(ObjectError::GetError {
                msg: format!("can not get object, status code: {}", resp.status()),
            }))
        }
    }

    fn get_object_acl<S>(&self, object_name: S) -> Result<String, OSSError>
    where
        S: AsRef<str>,
    {
        let object_name = object_name.as_ref();
        let mut params: HashMap<&str, Option<&str>> = HashMap::new();
        params.insert("acl", None);
        let result = String::from_utf8(self.get_object(object_name, None, Some(params))?)?;
        let mut reader = Reader::from_str(&result);
        reader.trim_text(true);
        let mut grant = String::new();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"Grant" => {
                    grant = reader.read_text(e.name())?.to_string();
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
        }

        Ok(grant)
    }

    fn put_object_from_file<S1, S2, S3, H, R>(
        &self,
        file: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>,
    {
        let (host, headers) =
            self.build_request(RequestType::Put, object_name, headers, resources)?;

        let buf = load_file(file)?;

        let resp = reqwest::blocking::Client::new()
            .put(host)
            .headers(headers)
            .body(buf)
            .send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(OSSError::Object(ObjectError::PutError {
                msg: format!("can not put object, status code: {}", resp.status()),
            }))
        }
    }

    fn put_object_from_buffer<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>,
    {
        let (host, headers) =
            self.build_request(RequestType::Put, object_name, headers, resources)?;

        let resp = reqwest::blocking::Client::new()
            .put(host)
            .headers(headers)
            .body(buf.to_owned())
            .send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(OSSError::Object(ObjectError::PutError {
                msg: format!("can not put object, status code: {}", resp.status()),
            }))
        }
    }

    fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), OSSError>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>,
    {
        let (host, mut headers) =
            self.build_request(RequestType::Put, object_name, headers, resources)?;
        headers.insert("x-oss-copy-source", src.as_ref().parse()?);

        let resp = reqwest::blocking::Client::new()
            .put(host)
            .headers(headers)
            .send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(OSSError::Object(ObjectError::CopyError {
                msg: format!("can not copy object, status code: {}", resp.status()),
            }))
        }
    }

    fn delete_object<S>(&self, object_name: S) -> Result<(), OSSError>
    where
        S: AsRef<str>,
    {
        let headers = HashMap::<String, String>::new();
        let (host, headers) =
            self.build_request(RequestType::Delete, object_name, Some(headers), None)?;

        let resp = reqwest::blocking::Client::new()
            .delete(host)
            .headers(headers)
            .send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(OSSError::Object(ObjectError::DeleteError {
                msg: format!("can not delete object, status code: {}", resp.status()),
            }))
        }
    }
}
