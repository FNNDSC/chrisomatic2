use std::collections::HashMap;
use std::rc::Rc;

use crate::dependency_tree::{Dag, DependencyTree, NodeIndex};
use crate::steps::*;
use chrisomatic_spec::*;
use chrisomatic_step::PendingStep;
use petgraph::acyclic::Acyclic;
use petgraph::data::Build;

pub fn plan(manifest: Manifest) -> DependencyTree<Rc<dyn PendingStep>> {
    let mut tree = TreeBuilder::new();
    let url = manifest.global.cube;
    let users: HashMap<_, _> = manifest
        .user
        .into_iter()
        .map(|(username, details)| add_steps_for_user(&mut tree, username, details, url.clone()))
        .collect();
    tree.into()
}

fn add_steps_for_user(
    tree: &mut TreeBuilder,
    username: Username,
    details: UserDetails,
    url: CubeUrl,
) -> (Username, NodeIndex) {
    let password = details.password.clone();
    let details = Rc::new(details);
    let exists = tree.add(
        UserExists {
            username: username.clone(),
            details: Rc::clone(&details),
            url: url.clone(),
        },
        vec![],
    );
    let auth_token = tree.add(
        UserGetAuthToken {
            username: username.clone(),
            password,
            url: url.clone(),
        },
        vec![exists],
    );
    let get_url = tree.add(
        UserGetUrl {
            username: username.clone(),
            details: Rc::clone(&details),
            url: url.clone(),
        },
        vec![auth_token],
    );
    let get_details = tree.add(
        UserGetDetails {
            url: url.clone(),
            username: username.clone(),
            details: Rc::clone(&details),
        },
        vec![get_url, auth_token],
    );
    tree.add(
        UserDetailsFinalize {
            username: username.clone(),
            details,
        },
        vec![get_details, auth_token],
    );
    (username, auth_token)
}

struct TreeBuilder(Dag<Rc<dyn PendingStep>>);

impl TreeBuilder {
    fn new() -> Self {
        Self(Acyclic::new())
    }

    fn add<T>(&mut self, pending_step: T, needs: Vec<NodeIndex>) -> NodeIndex
    where
        T: PendingStep + AsRef<dyn PendingStep> + 'static,
    {
        #[cfg(debug_assertions)]
        {
            // assert that `pending_step`'s dependencies are satisfied by
            // the nodes specified in `needs`
            let provided = self.provides(&needs);
            let needed = crate::dependency_spy::dependencies_of(&pending_step);
            debug_assert!(provided.is_superset(&needed));
        }
        let id = self.0.add_node(Rc::new(pending_step));
        for need in needs {
            self.0.try_add_edge(need, id, ()).unwrap();
        }
        id
    }

    /// Get a set of what dependencies can be provided by the specified nodes.
    #[cfg(debug_assertions)]
    fn provides<'a>(
        &self,
        needs: impl IntoIterator<Item = &'a NodeIndex>,
    ) -> std::collections::HashSet<chrisomatic_step::Dependency> {
        needs
            .into_iter()
            .map(|id| self.0.node_weight(*id).unwrap())
            .flat_map(|parent| crate::dependency_spy::provides_of(parent))
            .collect()
    }
}

impl From<TreeBuilder> for DependencyTree<Rc<dyn PendingStep>> {
    fn from(value: TreeBuilder) -> Self {
        DependencyTree(value.0)
    }
}

#[cfg(test)]
mod tests {

    use crate::dependency_spy::provides_of;

    use super::*;
    use chrisomatic_step::Dependency;
    use compact_str::CompactString;
    use rstest::*;

    #[rstest]
    fn test_add_steps_for_user(user: (Username, UserDetails), cube_url: CubeUrl) {
        let (username, details) = user;
        let mut tree = TreeBuilder::new();
        let (username, token_id) = add_steps_for_user(&mut tree, username, details, cube_url);
        let pending_step_for_token = tree.0.node_weight(token_id).unwrap();

        let provides = provides_of(pending_step_for_token);
        assert!(provides.contains(&Dependency::AuthToken(username.clone())));

        let start = DependencyTree::from(tree).start();
        let start_provides: HashSet<_> = start
            .into_iter()
            .map(|(_id, pending_step)| pending_step)
            .flat_map(provides_of)
            .collect();
        assert!(
            start_provides.contains(&Dependency::UserExists(username)),
            "Starting steps for user must provide a Dependency::UserExists, but does not, instead: {:?}",
            start_provides
        )
    }

    #[fixture]
    fn user() -> (Username, UserDetails) {
        let username = Username::new(CompactString::const_new("alice"));
        let details = UserDetails {
            password: "alice1234".to_string(),
            email: "alice.test@example.org".to_string(),
            groups: ["people", "pacs_users", "mri.team"]
                .map(|s| s.to_string())
                .into_iter()
                .collect(),
        };
        (username, details)
    }

    #[fixture]
    fn cube_url() -> CubeUrl {
        CubeUrl::try_new("https://example.com:12345/api/v1/").unwrap()
    }
}
