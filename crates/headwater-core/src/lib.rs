//! headwater's core, after the statement in boomdrivedev/blueprints
//! `design/headwater/requirements.md` and its model,
//! `design/headwater/models/reconstruction-1.md` with its rulings.
//!
//! The core is one derivation over one reading of a repository state and
//! what the lookups answered (model section 1):
//! `report = derive(reading, listings, options, now)`. Reading a
//! repository changes nothing in it (5), and the derivation reaches no
//! lookup and writes nothing. The seams are traits (model section 6): a
//! dependency description reads declarations, a release lookup lists an
//! identity's releases with the notes it carries, and the tiers
//! declaration binds tiers and lookups (C.7).

pub mod config;
pub mod declaration;
pub mod flake;
pub mod identity;
pub mod reading;
pub mod report;
pub mod seams;
pub mod version;

pub use config::{Binding, Config, ConfigError, TierDeclaration, TimberBinding};
pub use declaration::{Declaration, Dependency, Pin, Unread};
pub use identity::Identity;
pub use reading::{read_repository, Reading, RepositoryState};
pub use report::{derive, Options, Report};
pub use seams::{Listed, Listing, NotesAnswer, Query, ReleaseLookup};
pub use version::{level, order, Level, Order, TagScheme};
