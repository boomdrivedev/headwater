//! headwater's release lookups (C.3), each a profile of the core's
//! `ReleaseLookup` seam: GitHub releases, through GraphQL with the asker's
//! own `gh` sign-in, fifty repositories a query, the release bodies its
//! notes (C.4); and timber, the head of the channel the binding names, as
//! the host's timber serving process answers it (C.3, C.7).
//!
//! What either sends beyond the host is the identities asked about and
//! nothing else of the repository (37); neither keeps a credential (38).

pub mod github;
pub mod timber;

pub use github::{GhCli, GitHubReleases, GraphQl};
pub use timber::{HostServer, StatusSource, Timber};
