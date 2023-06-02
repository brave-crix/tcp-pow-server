use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::fmt;
use sha2;
use sha2::Digest;
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;
use colored::Colorize;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Hashcash {
    pub version: i32,
    pub zero_count: i32, //zero-byte count from leading
    pub date: i64,
    pub resource: String,
    pub ext: String,
    pub rand: i32,
    pub counter: i32
}

impl Hashcash {
    pub fn new(zero_count: i32, resource: String) -> Self {
        Hashcash {
            version: 1,
            zero_count,
            date: Utc::now().timestamp(),
            resource: resource.to_string(),
            ext: "".to_string(),
            rand: rand::random(),
            counter: 0,
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        let data = self.to_string();
        let mut hasher = sha2::Sha256::new();
        hasher.update(data.as_bytes());
        hasher.finalize().as_slice().to_vec()
    }

    pub fn is_hash_valid(hash: Vec<u8>, zero_count:i32) -> bool{
        let mut leading_zeros = 0;
        for byte in hash.iter() {
            if byte & 0xF0 == 0 { // Check if Hi-Hex is zero
                leading_zeros += 1;
            } else {
                break;
            }

            if byte & 0xF == 0 { // Check if Lo-Hex is zero
                leading_zeros += 1;
            } else {
                break;
            }
            log::debug!("Hash: {:x?}", hash.as_slice());
        }
        leading_zeros >= zero_count
    }


    pub fn is_valid(&self) -> bool{
        let byte_code = self.hash();
        Self::is_hash_valid(byte_code, self.zero_count)
    }

    pub fn try_work(&mut self, max_iter: i32) -> Result<(), Box<dyn Error>> {
        while self.counter <= max_iter {
            self.counter += 1;
            if self.is_valid() {
                break;
            }
        }
        log::warn!("Tried {} times to solve", format!("{}", self.counter).red().italic());
        if self.counter > max_iter {
            return Err("Max iterations exceeded".into());
        }
        Ok(())
    }
}

impl fmt::Display for Hashcash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}:{}:{}:{}:{}", self.version, self.zero_count, self.date, self.resource, general_purpose::STANDARD.encode(self.rand.to_string()), self.ext, general_purpose::STANDARD.encode(self.counter.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_hash_cash() {
        let hashcash = Hashcash::new(3, "Word of Wisdom".to_string());
        assert_eq!(hashcash.zero_count, 3);
        assert_eq!(hashcash.resource, "Word of Wisdom".to_string());
    }
    #[test]
    fn test_hash_valid() {
        let hash = vec![0, 16, 0, 0, 0];
        assert!(Hashcash::is_hash_valid(hash.clone(), 1));
        assert!(Hashcash::is_hash_valid(hash.clone(), 2));
        assert_eq!(Hashcash::is_hash_valid(hash.clone(), 3), false);
        assert_eq!(Hashcash::is_hash_valid(hash, 4), false);
    }

    #[test]
    fn test_valid_hash() {
        let mut hashcash = Hashcash::new(4, "Word of Wisdom".to_string());
        let max_iter = 10000;
        if hashcash.is_valid() {
            
        } else {
            match hashcash.try_work(max_iter) {
                Ok(_) => {
                    assert!(hashcash.is_valid());
                    assert!(hashcash.counter > 0);
                }
                Err(_e) => {
                    assert!(hashcash.counter > max_iter)
                }
            }    
        }
    }
}
