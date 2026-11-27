use super::*;

/// The migrations of a run, stage by stage. `Order` keeps the stages sorted
/// whatever the order they are filled in, and what a stage holds keeps the
/// order it was given in.
#[derive(Default)]
pub(crate) struct Stages {
    files: BTreeMap<Order, Plan>,
}

impl Stages {
    /// The files of `stage`, to add to.
    pub(crate) fn files_of(&mut self, stage: Stage) -> &mut Plan {
        self.files.entry(Order::of(stage)).or_default()
    }

    /// The plan of the whole run, its stages one after the other. Each file
    /// is written as waiting, and says which migration it starts from: the
    /// last one about its table or its enum, among the `settled` ones and
    /// those the run puts before it.
    pub(crate) fn in_order(self, settled: &[String]) -> Plan {
        let mut plan = Plan::default();
        let mut before: Vec<String> = settled.to_vec();
        for stage in self.files.into_values() {
            for ((path, content), module) in stage.files.into_iter().zip(stage.lib_modules) {
                plan.files
                    .push((path, waiting_migrate(content, &module, &before)));
                before.push(module.clone());
                plan.lib_modules.push(module);
            }
        }
        plan
    }
}
