use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "msg")]
#[serde(rename_all = "lowercase")]
pub enum Message {
    Ping,
    Pong,
    Connect { version: String, support: Vec<String> },
    Method { id: String, method: String, params: Vec<Value> },
    Result(MethodResult),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
pub enum MethodResult {
    Result { id: String, result: Value },
    Error { id: String, error: Value },
}


#[test]
fn test_method_format() {

    fn check_message(msg: &Message) {

        let s = serde_json::to_string(msg).unwrap();
        let msg2: Message = serde_json::from_str(&s).unwrap();
        assert_eq!(msg, &msg2);
    }

    check_message(&Message::Result( 
        MethodResult::Result {
            id: "lolilol".to_string(),
            result: Value::String("burp".to_string()),
        }
    ));

    check_message(&Message::Result(
        MethodResult::Error {
            id: "lulz".to_string(),
            error: Value::Bool(true),
        }
    ));

}