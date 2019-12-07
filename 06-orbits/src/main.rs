/// So... it seems that trees and graphs are tricky in Rust. Storing references from each node to
/// the parent means that the parent fields needs to be something like a RefCell<Option<Rc<Body>>>
/// which gets messy quickly.
///
/// Instead, I'm storing all the nodes in a System, which contains a HashMap of the nodes, and
/// methods where a Body access its parent needs to be provided a reference to the System.
///
/// I'm also pretty sure my heavy use of String::from and String.clone is not idiomatic at all.
/// Replacing them with &str throws up all kinds of lifetime issues which I'm not yet sure how to
/// resolve.
use std::collections::HashMap;
use std::fs;

/// Contains all the bodies in the system.
struct System {
    bodies: HashMap<String, Body>,
}

impl System {
    /// Reads the parsed orbits data and constructs a System and the bodies which belong.
    fn new_with_data<'a>(data: Vec<(&'a str, &'a str)>) -> System {
        let mut bodies = HashMap::with_capacity(data.len());

        for (center, orbiter) in &data {
            if !bodies.contains_key(*center) {
                bodies.insert(String::from(*center), Body::new(String::from(*center)));
            }

            if !bodies.contains_key(*orbiter) {
                bodies.insert(
                    String::from(*orbiter),
                    Body::new_with_parent(String::from(*orbiter), String::from(*center)),
                );
            } else {
                bodies
                    .get_mut(&String::from(*orbiter))
                    .unwrap()
                    .set_parent(String::from(*center));
            }
        }

        System { bodies }
    }

    /// Find the number of transfer orbits required to move from orbiting the `source` body to the
    /// `target`. This is done by building a HashMap where each key is a parent key of the source
    /// Body and each each value the number of transfer orbits required, then iterating through the
    /// target Body parents until a common ancestor is found.
    fn transfer_distance(&self, source: &Body, target: &Body) -> Option<usize> {
        let mut source_parents = HashMap::new();
        let mut parent = Some(source);
        let mut distance: usize = 0;

        while let Some(p) = parent {
            source_parents.insert(p.name.clone(), distance);

            distance += 1;
            parent = p.parent(self);
        }

        let mut target_pointer = Some(target);
        let mut distance = 0;

        while let Some(t) = target_pointer {
            if source_parents.contains_key(&t.name) {
                return Some(source_parents.get(&t.name).unwrap() + distance);
            }

            distance += 1;
            target_pointer = t.parent(self);
        }

        None
    }
}

/// Descibes a body in the solar system which directly orbits zero or one other bodies.
#[derive(Debug)]
struct Body {
    name: String,
    parent_key: Option<String>,
}

impl Body {
    /// Creates a new Body without a parent.
    fn new(name: String) -> Body {
        Body {
            name,
            parent_key: None,
        }
    }

    /// Creates a body, which orbits the parent identified by the `parent_key`.
    fn new_with_parent(name: String, parent_key: String) -> Body {
        Body {
            name,
            parent_key: Some(parent_key),
        }
    }

    /// Returns the Body which is orbited by this Body. Returns None if the Body has no parent.
    fn parent<'a>(&self, system: &'a System) -> Option<&'a Body> {
        match &self.parent_key {
            None => None,
            Some(parent_key) => system.bodies.get(parent_key),
        }
    }

    /// Calculates the number of direct and indirect orbits. The body orbits its parent directly,
    /// and the parent of its parents (and so on...) indirectly.
    fn num_orbits(&self, system: &System) -> usize {
        match &self.parent_key {
            Some(_) => 1 + self.parent(&system).unwrap().num_orbits(&system),
            None => 0,
        }
    }

    fn set_parent(&mut self, parent_key: String) {
        self.parent_key = Some(parent_key);
    }
}

fn main() {
    let data = fs::read_to_string("data/orbits.txt").unwrap();
    let data = data.trim();

    let data: Vec<(&str, &str)> = data
        .lines()
        .map(|line| {
            let mut parts = line.split(")");
            (parts.next().unwrap(), parts.next().unwrap())
        })
        .collect();

    let system = System::new_with_data(data);

    println!(
        "Total direct and indirect orbits: {:?}",
        system
            .bodies
            .values()
            .fold(0, |memo, body| memo + body.num_orbits(&system))
    );

    println!(
        "Transfer distance: {}",
        system.transfer_distance(
            system.bodies.get("YOU").unwrap(),
            system.bodies.get("SAN").unwrap(),
        )
        .expect("Failed to calculate YOU->SAN transfer distance.")
    );
}
