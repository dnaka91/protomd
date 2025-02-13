use std::{collections::HashMap, path::Path, rc::Rc};

use parking_lot::Mutex;
use protox::file::FileResolver;

pub struct CachingFileResolver<T> {
    inner: Rc<T>,
    cache: Rc<Mutex<HashMap<String, protox::file::File>>>,
}

impl<T> CachingFileResolver<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Rc::new(inner),
            cache: Rc::default(),
        }
    }
}

impl<T> Clone for CachingFileResolver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            cache: Rc::clone(&self.cache),
        }
    }
}

impl<T: FileResolver> FileResolver for CachingFileResolver<T> {
    fn resolve_path(&self, path: &Path) -> Option<String> {
        self.inner.resolve_path(path)
    }

    fn open_file(&self, name: &str) -> Result<protox::file::File, protox::Error> {
        let mut cache = self.cache.lock();
        if let Some(entry) = cache.get(name) {
            Ok(entry.clone())
        } else {
            let file = self.inner.open_file(name)?;
            cache.insert(name.to_owned(), file.clone());
            Ok(file)
        }
    }
}
