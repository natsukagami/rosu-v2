use crate::{
    model::{
        matches_::{MatchList, OsuMatch},
        Cursor,
    },
    request::{serialize::maybe_cursor, Pending, Query, Request},
    routing::Route,
    Osu,
};

use serde::Serialize;

#[cfg(feature = "cache")]
use futures::TryFutureExt;

/// Get an [`OsuMatch`](crate::model::matches::OsuMatch) by its id
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Serialize)]
pub struct GetMatch<'a> {
    #[serde(skip)]
    fut: Option<Pending<'a, OsuMatch>>,
    #[serde(skip)]
    osu: &'a Osu,
    #[serde(skip)]
    match_id: u32,
    after: Option<u64>,
    before: Option<u64>,
    limit: Option<usize>,
}

impl<'a> GetMatch<'a> {
    #[inline]
    pub(crate) fn new(osu: &'a Osu, match_id: u32) -> Self {
        Self {
            fut: None,
            osu,
            match_id,
            after: None,
            before: None,
            limit: None,
        }
    }

    /// Get the match state containing only events after the given event id.
    ///
    /// Note: The given event id won't be included.
    #[inline]
    pub fn after(mut self, after: u64) -> Self {
        self.after.replace(after);

        self
    }

    /// Get the match state containing only events before the given event id.
    ///
    /// Note: The given event id won't be included.
    #[inline]
    pub fn before(mut self, before: u64) -> Self {
        self.before.replace(before);

        self
    }

    /// Get the match state after at most `limit` many new events.
    #[inline]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit.replace(limit);

        self
    }

    fn start(&mut self) -> Pending<'a, OsuMatch> {
        let query = Query::encode(self);

        let route = Route::GetMatch {
            match_id: Some(self.match_id),
        };

        let req = Request::with_query(route, query);
        let osu = self.osu;
        let fut = osu.request::<OsuMatch>(req);

        #[cfg(feature = "cache")]
        let fut = fut.inspect_ok(move |osu_match| {
            for user in osu_match.users.values() {
                osu.update_cache(user.user_id, &user.username);
            }
        });

        Box::pin(fut)
    }
}

poll_req!(GetMatch => OsuMatch);

/// Get a [`MatchList`](crate::model::matches::MatchList) containing all
/// currently open multiplayer lobbies.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Serialize)]
pub struct GetMatches<'a> {
    #[serde(skip)]
    fut: Option<Pending<'a, MatchList>>,
    #[serde(skip)]
    osu: &'a Osu,
    #[serde(flatten, serialize_with = "maybe_cursor")]
    cursor: Option<Cursor>,
}

impl<'a> GetMatches<'a> {
    #[inline]
    pub(crate) fn new(osu: &'a Osu) -> Self {
        Self {
            fut: None,
            osu,
            cursor: None,
        }
    }

    #[inline]
    pub(crate) fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor.replace(cursor);

        self
    }

    fn start(&mut self) -> Pending<'a, MatchList> {
        let query = Query::encode(self);
        let req = Request::with_query(Route::GetMatch { match_id: None }, query);

        Box::pin(self.osu.request(req))
    }
}

poll_req!(GetMatches => MatchList);