#[cfg(feature = "diagnostics")]
use super::{QueryAccess, QueryLayout};

#[cfg(feature = "diagnostics")]
use comet_diagnostics::Diagnostics;

#[cfg(feature = "diagnostics")]
use serde::Serialize;

#[cfg(feature = "diagnostics")]
use std::{any::type_name, time::Instant};

#[cfg(feature = "diagnostics")]
pub(crate) struct QueryDiagnostics {
    output: Diagnostics,
    started: Instant,
    planning_started: Option<Instant>,
    record: QueryDiagnosticRecord,
}

#[cfg(feature = "diagnostics")]
#[derive(Serialize)]
struct QueryDiagnosticRecord {
    data: &'static str,
    filter: &'static str,
    mutable: bool,
    construction_ns: u128,
    planning_ns: u128,
    execution_ns: u128,
    archetypes_total: usize,
    archetypes_inspected: usize,
    cache_hits: usize,
    cache_misses: usize,
    component_targets: usize,
    amount_targets: usize,
    entity_selectors: usize,
    row_selectors: usize,
    planned_combinations: usize,
    accesses: usize,
    planned_rows: usize,
    filtered_accesses: usize,
    entity_selection_rows: usize,
    row_selection_rows: usize,
    rows_considered: usize,
    rows_rejected: usize,
    rows_yielded: usize,
}

#[cfg(feature = "diagnostics")]
impl QueryDiagnostics {
    pub(crate) fn new<Data, Filter>(mutable: bool, archetypes_total: usize) -> Self {
        Self {
            output: Diagnostics::from_env(),
            started: Instant::now(),
            planning_started: None,
            record: QueryDiagnosticRecord {
                data: type_name::<Data>(),
                filter: type_name::<Filter>(),
                mutable,
                construction_ns: 0,
                planning_ns: 0,
                execution_ns: 0,
                archetypes_total,
                archetypes_inspected: 0,
                cache_hits: 0,
                cache_misses: 0,
                component_targets: 0,
                amount_targets: 0,
                entity_selectors: 0,
                row_selectors: 0,
                planned_combinations: 0,
                accesses: 0,
                planned_rows: 0,
                filtered_accesses: 0,
                entity_selection_rows: 0,
                row_selection_rows: 0,
                rows_considered: 0,
                rows_rejected: 0,
                rows_yielded: 0,
            },
        }
    }

    pub(crate) fn begin_planning(&mut self) {
        self.planning_started = Some(Instant::now());
    }

    pub(crate) fn finish_build(
        &mut self,
        layout: &QueryLayout,
        accesses: &[QueryAccess],
        row_selectors: usize,
    ) {
        self.record.planning_ns = self
            .planning_started
            .take()
            .map_or(0, |started| started.elapsed().as_nanos());
        self.record.construction_ns = self.started.elapsed().as_nanos();
        self.record.component_targets = layout.components.len();
        self.record.amount_targets = layout.amounts.len();
        self.record.entity_selectors = layout.entity_ranges.len();
        self.record.row_selectors = row_selectors;
        self.record.accesses = accesses.len();
        self.record.planned_rows = accesses.iter().map(|access| access.len).sum();
    }

    pub(crate) fn inspect_archetype(&mut self) {
        self.record.archetypes_inspected += 1;
    }

    pub(crate) fn cache_hit(&mut self) {
        self.record.cache_hits += 1;
    }

    pub(crate) fn cache_miss(&mut self) {
        self.record.cache_misses += 1;
    }

    pub(crate) fn planned_combinations(&mut self, amount: usize) {
        self.record.planned_combinations += amount;
    }

    pub(crate) fn filtered_access(&mut self) {
        self.record.filtered_accesses += 1;
    }

    pub(crate) fn entity_selection_row(&mut self) {
        self.record.entity_selection_rows += 1;
    }

    pub(crate) fn row_selection_row(&mut self) {
        self.record.row_selection_rows += 1;
    }

    pub(crate) fn row_considered(&mut self) {
        self.record.rows_considered += 1;
    }

    pub(crate) fn row_rejected(&mut self) {
        self.record.rows_rejected += 1;
    }

    pub(crate) fn row_yielded(&mut self) {
        self.record.rows_yielded += 1;
    }

    pub(crate) fn publish(&mut self) {
        self.record.execution_ns = self
            .started
            .elapsed()
            .as_nanos()
            .saturating_sub(self.record.construction_ns);
        self.output.publish("ecs", "query", &self.record);
    }
}

#[cfg(not(feature = "diagnostics"))]
#[derive(Default)]
pub(crate) struct QueryDiagnostics;

#[cfg(not(feature = "diagnostics"))]
impl QueryDiagnostics {
    pub(crate) fn new<Data, Filter>(_: bool, _: usize) -> Self {
        Self
    }

    pub(crate) fn begin_planning(&mut self) {}
    pub(crate) fn finish_build<T, U>(&mut self, _: &T, _: &[U], _: usize) {}
    pub(crate) fn inspect_archetype(&mut self) {}
    pub(crate) fn cache_hit(&mut self) {}
    pub(crate) fn cache_miss(&mut self) {}
    pub(crate) fn planned_combinations(&mut self, _: usize) {}
    pub(crate) fn filtered_access(&mut self) {}
    pub(crate) fn entity_selection_row(&mut self) {}
    pub(crate) fn row_selection_row(&mut self) {}
    pub(crate) fn row_considered(&mut self) {}
    pub(crate) fn row_rejected(&mut self) {}
    pub(crate) fn row_yielded(&mut self) {}
    pub(crate) fn publish(&mut self) {}
}
