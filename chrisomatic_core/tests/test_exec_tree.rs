use std::rc::Rc;

use chrisomatic_core::{DependencyTree, Outcome, exec_tree};
use chrisomatic_spec::Username;
use chrisomatic_step::*;
use compact_str::ToCompactString;
use futures_lite::StreamExt;
use nonempty::{NonEmpty, nonempty};
use petgraph::{acyclic::Acyclic, data::Build, prelude::StableDiGraph};
use reqwest::{Method, Request, Url};
use warp::Filter;

/// This tests asserts that [exec_tree] runs steps in topological order.
/// It creates a dependency tree consisting of steps which send simple HTTP
/// requests to an HTTP server on a random port running in a background task.
#[tokio::test]
async fn test_exec_tree() {
    let addr = local_addr();
    let port = addr.port();
    let server_task = tokio::spawn(test_server(addr));

    let mut dag: Acyclic<StableDiGraph<Rc<dyn PendingStep>, ()>> = Acyclic::new();
    /*
       a   d
       |   |
       b   e
      /|   |
     u c   f
      /
     v
    */
    let a = dag.add_node(Rc::new(TestPendingStep { data: 'a', port }));
    let b = dag.add_node(Rc::new(TestPendingStep { data: 'b', port }));
    let c = dag.add_node(Rc::new(TestPendingStep { data: 'c', port }));
    let d = dag.add_node(Rc::new(TestPendingStep { data: 'd', port }));
    let e = dag.add_node(Rc::new(TestPendingStep { data: 'e', port }));
    let f = dag.add_node(Rc::new(TestPendingStep { data: 'f', port }));
    let u = dag.add_node(Rc::new(AlwaysFulfilledPendingStep));
    let v = dag.add_node(Rc::new(AlwaysFulfilledPendingStep));
    dag.try_add_edge(a, b, ()).unwrap();
    dag.try_add_edge(b, c, ()).unwrap();
    dag.try_add_edge(d, e, ()).unwrap();
    dag.try_add_edge(e, f, ()).unwrap();
    dag.try_add_edge(b, u, ()).unwrap();
    dag.try_add_edge(c, v, ()).unwrap();

    let outcomes: Vec<Outcome> = exec_tree(reqwest::Client::new(), DependencyTree::new(dag))
        .collect()
        .await;
    assert_eq!(outcomes.len(), 8);
    let index_of = |x: char| {
        outcomes
            .iter()
            .map(|outcome| &outcome.target)
            .enumerate()
            .find(|(_, target)| {
                *target == &Dependency::UserExists(Username::new(x.to_compact_string()))
            })
            .map(|(i, _)| i)
            .unwrap()
    };
    let index_a = index_of('a');
    let index_b = index_of('b');
    let index_c = index_of('c');
    let index_d = index_of('d');
    let index_e = index_of('e');
    let index_f = index_of('f');

    assert!(
        index_a < index_b,
        "step 'a' ran after step 'b' but 'b' depended on 'a'"
    );
    assert!(
        index_b < index_c,
        "step 'b' ran after step 'c' but 'c' depended on 'b'"
    );
    assert!(
        index_d < index_e,
        "step 'd' ran after step 'e' but 'e' depended on 'd'"
    );
    assert!(
        index_e < index_f,
        "step 'e' ran after step 'f' but 'f' depended on 'e'"
    );

    server_task.abort();
}

#[derive(Copy, Clone, Debug)]
struct TestPendingStep {
    data: char,
    port: u16,
}

#[derive(Copy, Clone, Debug)]
struct AlwaysFulfilledPendingStep;

impl PendingStep for AlwaysFulfilledPendingStep {
    fn build(&self, map: &dyn DependencyMap) -> PendingStepResult {
        if map.contains_key(&Dependency::UserExists("a".into())) {
            Ok(None)
        } else {
            Ok(Some(Rc::new(ShouldNeverRunStep)))
        }
    }
}

struct ShouldNeverRunStep;

impl Step for ShouldNeverRunStep {
    fn search(&self) -> reqwest::Request {
        unimplemented!()
    }

    fn deserialize(&self, _: bytes::Bytes) -> serde_json::Result<Check> {
        unimplemented!()
    }

    fn provides(&self) -> NonEmpty<Dependency> {
        nonempty![Dependency::UserExists("a".into())]
    }
}

impl TestPendingStep {
    fn dummy_username(&self) -> Username {
        Username::new(self.data.to_compact_string())
    }

    fn url(&self) -> Url {
        Url::parse(&format!("http://localhost:{}/dbl/{}", self.port, self.data)).unwrap()
    }
}

impl PendingStep for TestPendingStep {
    fn build(&self, _: &dyn DependencyMap) -> PendingStepResult {
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

fn local_addr() -> std::net::SocketAddr {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

async fn test_server(addr: std::net::SocketAddr) {
    let api = warp::path!("dbl" / String).map(|path: String| format!("{}{}", &path, &path));
    warp::serve(api).run(addr).await
}
