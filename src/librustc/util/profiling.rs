use std::borrow::Cow;
use std::error::Error;
use std::mem::{self, Discriminant};
use std::process;
use std::thread::ThreadId;
use std::u32;

use crate::ty::query::QueryName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProfileCategory {
    Parsing,
    Expansion,
    TypeChecking,
    BorrowChecking,
    Codegen,
    Linking,
    Other,
}

bitflags! {
    struct EventFilter: u32 {
        const GENERIC_ACTIVITIES = 1 << 0;
        const QUERY_PROVIDERS    = 1 << 1;
        const QUERY_CACHE_HITS   = 1 << 2;
        const QUERY_BLOCKED      = 1 << 3;
        const INCR_CACHE_LOADS   = 1 << 4;

        const DEFAULT = Self::GENERIC_ACTIVITIES.bits |
                        Self::QUERY_PROVIDERS.bits |
                        Self::QUERY_BLOCKED.bits |
                        Self::INCR_CACHE_LOADS.bits;

        // empty() and none() aren't const-fns unfortunately
        const NONE = 0;
        const ALL  = !Self::NONE.bits;
    }
}

const EVENT_FILTERS_BY_NAME: &[(&str, EventFilter)] = &[
    ("none", EventFilter::NONE),
    ("all", EventFilter::ALL),
    ("generic-activity", EventFilter::GENERIC_ACTIVITIES),
    ("query-provider", EventFilter::QUERY_PROVIDERS),
    ("query-cache-hit", EventFilter::QUERY_CACHE_HITS),
    ("query-blocked" , EventFilter::QUERY_BLOCKED),
    ("incr-cache-load", EventFilter::INCR_CACHE_LOADS),
];

fn thread_id_to_u64(tid: ThreadId) -> u64 {
    unsafe { mem::transmute::<ThreadId, u64>(tid) }
}

pub struct SelfProfiler {
    event_filter_mask: EventFilter,
}

impl SelfProfiler {
    pub fn new(event_filters: &Option<Vec<String>>) -> Result<SelfProfiler, Box<dyn Error>> {
        let filename = format!("pid-{}.rustc_profile", process::id());
        let path = std::path::Path::new(&filename);
        let mut event_filter_mask = EventFilter::empty();

        if let Some(ref event_filters) = *event_filters {
            let mut unknown_events = vec![];
            for item in event_filters {
                if let Some(&(_, mask)) = EVENT_FILTERS_BY_NAME.iter()
                                                               .find(|&(name, _)| name == item) {
                    event_filter_mask |= mask;
                } else {
                    unknown_events.push(item.clone());
                }
            }

            // Warn about any unknown event names
            if unknown_events.len() > 0 {
                unknown_events.sort();
                unknown_events.dedup();

                warn!("Unknown self-profiler events specified: {}. Available options are: {}.",
                    unknown_events.join(", "),
                    EVENT_FILTERS_BY_NAME.iter()
                                         .map(|&(name, _)| name.to_string())
                                         .collect::<Vec<_>>()
                                         .join(", "));
            }
        } else {
            event_filter_mask = EventFilter::DEFAULT;
        }

        Ok(SelfProfiler {
            event_filter_mask,
        })
    }

    pub fn register_query_name(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn start_activity(
        &self,
        _label: impl Into<Cow<'static, str>>,
    ) {
    }

    #[inline]
    pub fn end_activity(
        &self,
        _label: impl Into<Cow<'static, str>>,
    ) {
    }

    #[inline]
    pub fn record_query_hit(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn start_query(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn end_query(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn incremental_load_result_start(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn incremental_load_result_end(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn query_blocked_start(&self, _query_name: QueryName) {
    }

    #[inline]
    pub fn query_blocked_end(&self, _query_name: QueryName) {
    }
}
