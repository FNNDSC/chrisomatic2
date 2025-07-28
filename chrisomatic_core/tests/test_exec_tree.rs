use std::rc::Rc;

use chrisomatic_core::exec_tree;
use chrisomatic_spec::Username;
use chrisomatic_step::*;
use compact_str::{CompactString, ToCompactString};
use nonempty::{NonEmpty, nonempty};
use reqwest::{Method, Request, Url};

#[tokio::test]
async fn test_exec_tree() {
    assert_eq!(1, 1)
}

#[derive(Copy, Clone, Debug)]
struct TestPendingStep {
    data: char,
    port: u16,
}

impl TestPendingStep {
    fn dummy_username(&self) -> Username {
        Username::new(self.data.to_compact_string())
    }

    fn url(&self) -> Url {
        Url::parse(&format!("http://localhost:{}/{}", self.port, self.data)).unwrap()
    }
}

impl PendingStep for TestPendingStep {
    fn build(&self, map: &dyn DependencyMap) -> PendingStepResult {
        Ok(Some(Rc::new(TestStep(self.clone()))))
    }
}

struct TestStep(TestPendingStep);

impl Step for TestStep {
    fn search(&self) -> reqwest::Request {
        Request::new(Method::GET, self.0.url())
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check> {
        let data = String::from_utf8(body.into()).unwrap();
        let out = vec![(Dependency::UserExists(self.0.dummy_username()), data)];
        Ok(Check::Exists(out))
    }

    fn provides(&self) -> NonEmpty<Dependency> {
        nonempty![Dependency::UserExists(self.0.dummy_username())]
    }
}
