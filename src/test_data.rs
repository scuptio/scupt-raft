pub mod tests {
    use std::fs;
    use std::fs::read_to_string;
    use std::path::PathBuf;
    use std::str::FromStr;

    use serde::de::DeserializeOwned;
    use serde::Serialize;

    use crate::test_path::tests::test_data_path;

    fn from_json<R: DeserializeOwned>(s: String) -> R {
        let r: serde_json::Result<R> = serde_json::from_str(s.as_str());
        match r {
            Err(_t) => {
                panic!("error serde, may be need to update the data json {:?}", s);
            }
            Ok(r) => { return r }
        }
    }
    pub fn test_data<Input: DeserializeOwned, Result: DeserializeOwned>(path: String) -> Vec<(Input, Result)> {
        let path = test_data_path(path).unwrap();
        let mut vec = vec![];
        let paths = fs::read_dir(path).unwrap();
        for entry in paths {
            let path = entry.unwrap().path();
            let json = read_to_string(path).unwrap();
            let (input, result) = from_json(json);
            vec.push((input, result));
        }
        vec
    }

    pub fn gen_test_json<Input: Serialize, Result: Serialize>(
        path: &str,
        input: Input,
        result: Result,
        enable: bool,
    ) {
        if enable {
            let mut path_buf = PathBuf::from_str(path).unwrap();
            let _s = (input, result);
            let json = serde_json::to_string_pretty(&_s).unwrap();
            let md5 = md5::compute(json.as_bytes());
            if !path_buf.exists() {
                fs::create_dir_all(path_buf.clone()).unwrap();
            }
            path_buf.push(format!("{:x}.json", md5));
            fs::write(path_buf, json).unwrap();
        }
    }
}

