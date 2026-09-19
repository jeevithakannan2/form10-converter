pub(crate) const MONTHS: [&str; 12] = [
    "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC", "JAN", "FEB", "MAR",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub code: String,
    pub name: String,
    pub active_months: [bool; 12],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceData {
    pub sheet_name: String,
    pub members: Vec<Member>,
    pub financial_year: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rates {
    pub old_member: f64,
    pub old_society: f64,
    pub old_union: f64,
    pub new_member: f64,
    pub new_society: f64,
    pub new_union: f64,
    // 0 means all old rates. 1 means new rates from April. 12 means new rates from March.
    pub new_from_month: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub financial_year: String,
    pub dcmpu: String,
    pub district: String,
    pub society: String,
    pub society_code: String,
    pub rates: Rates,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ContributionKind {
    Member,
    Society,
    Union,
}

impl Rates {
    pub(crate) fn contribution(&self, active_months: &[bool; 12], kind: ContributionKind) -> f64 {
        active_months
            .iter()
            .enumerate()
            .filter(|(_, active)| **active)
            .map(|(month, _)| {
                let new_rate_applies =
                    self.new_from_month > 0 && month + 1 >= self.new_from_month as usize;
                match (kind, new_rate_applies) {
                    (ContributionKind::Member, false) => self.old_member,
                    (ContributionKind::Society, false) => self.old_society,
                    (ContributionKind::Union, false) => self.old_union,
                    (ContributionKind::Member, true) => self.new_member,
                    (ContributionKind::Society, true) => self.new_society,
                    (ContributionKind::Union, true) => self.new_union,
                }
            })
            .sum()
    }
}
