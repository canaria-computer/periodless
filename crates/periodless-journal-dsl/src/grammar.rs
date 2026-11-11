use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "journal_entry.pest"]
pub(crate) struct JournalEntryGrammar;
