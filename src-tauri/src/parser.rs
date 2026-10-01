//! Turns a parsed PCM database into the scouting model the UI works with.
use crate::cdb::Cdb;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

const FREE_AGENT_TEAM_ID: i32 = 119;

// ─── Output types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct TopSkill {
    pub key: String,
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Cyclist {
    pub id: i32,
    pub name: String,
    pub firstname: String,
    pub lastname: String,
    pub team_id: i32,
    pub team: String,
    pub team_short: String,
    pub division: String,
    pub nationality: String,
    pub continent: String,
    pub iso: String,
    pub flag: String,
    pub birthdate: String,
    pub age: i32,
    /// PCM lets you sign a rider from the season in which they turn 18 (the month doesn't matter).
    pub signable: bool,
    /// First season the rider can be signed (birth year + 18).
    pub signable_from: i32,
    pub rider_type: String,
    pub rider_type_id: i32,
    pub size: i32,
    pub weight: i32,
    pub current_ability: f32,
    pub potential: f32,
    pub growth: f32,
    pub skill_average: f32,
    pub skill_ceiling: f32,
    pub peak_gap: i32,
    pub specialty_rating: i32,
    pub scout_grade: String,
    pub free_agent: bool,
    pub is_mine: bool,
    pub flat: i32,        pub flat_p: i32,
    pub mountain: i32,    pub mountain_p: i32,
    pub med_mtn: i32,     pub med_mtn_p: i32,
    pub downhill: i32,    pub downhill_p: i32,
    pub cobble: i32,      pub cobble_p: i32,
    pub timetrial: i32,   pub timetrial_p: i32,
    pub prologue: i32,    pub prologue_p: i32,
    pub sprint: i32,      pub sprint_p: i32,
    pub acceleration: i32,pub acceleration_p: i32,
    pub endurance: i32,   pub endurance_p: i32,
    pub resistance: i32,  pub resistance_p: i32,
    pub recuperation: i32,pub recuperation_p: i32,
    pub hill: i32,        pub hill_p: i32,
    pub baroudeur: i32,   pub baroudeur_p: i32,
    pub top_skills: Vec<TopSkill>,
    /// Monthly wage in euros (0 when unsigned).
    pub wage: i32,
    pub contract_start: i32,
    /// Last season of the current contract (0 when unsigned).
    pub contract_end: i32,
    pub popularity: f32,
    pub wins: i32,
    pub tour_rating: i32,
    pub classic_rating: i32,
    pub injured: bool,
    pub will_retire: bool,
    /// Listed on the in-game transfer market (free agent or contract ending this season).
    pub on_market: bool,
    /// Number of scout reports filed on this rider by any team.
    pub scout_reports: i32,
    /// Whether one of your own scouts has reported on this rider.
    pub my_report: bool,
    pub scout_report_date: String,
    /// Best category estimate (stars) from the report shown.
    pub scout_estimate: f32,
    pub scout_tour_potential: f32,
    pub scout_mountain_potential: f32,
    pub scout_timetrial_potential: f32,
    pub scout_sprint_potential: f32,
    pub scout_ardennes_potential: f32,
    pub scout_cobble_potential: f32,
    pub scout_flat_potential: f32,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub short: String,
    pub abbreviation: String,
    pub country_iso: String,
    pub country_name: String,
    pub flag: String,
    pub color1: String,
    pub color2: String,
    pub division_id: i32,
    pub division: String,
    pub tier: i32,
    /// Season budget granted by the sponsor (`DYN_team.value_i_budget`).
    pub budget: i32,
    pub sponsor_id: i32,
    /// Row id in `DYN_team_sponsor` (0 when the team has no sponsor deal).
    pub sponsor_deal_id: i32,
    pub sponsor: String,
    pub sponsor_budget: i32,
    pub sponsor_budget_next: i32,
    pub sponsor_contract_end: i32,
    /// Monthly rider wages.
    pub payroll: i32,
    /// Monthly staff wages (coaches, physicians, scouts).
    pub staff_payroll: i32,
    pub riders: i32,
    pub avg_ca: f32,
    pub top_ca: f32,
    pub evaluation: f32,
    pub is_mine: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Staff {
    pub id: i32,
    pub role: String,
    pub name: String,
    pub wage: i32,
    pub contract_end: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct BrandDeal {
    pub id: i32,
    pub brand: String,
    pub category: String,
    pub budget: i32,
    pub year: i32,
    pub years_left: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct SponsorOffer {
    pub sponsor_id: i32,
    pub sponsor: String,
    pub budget: i32,
    pub duration: i32,
    pub contact_date: String,
    pub deadline: String,
    pub state: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct LedgerEntry {
    pub id: i32,
    pub date: String,
    pub amount: i32,
    pub category: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Finance {
    pub team_id: i32,
    /// Cash balance shown in-game (`GAM_career_data` → `SOLDE`).
    pub balance: f64,
    pub balance_editable: bool,
    pub season_budget: i32,
    pub sponsor_id: i32,
    pub sponsor_deal_id: i32,
    pub sponsor: String,
    pub sponsor_budget: i32,
    pub sponsor_budget_next: i32,
    pub sponsor_contract_start: i32,
    pub sponsor_contract_end: i32,
    pub monthly_rider_wages: i32,
    pub monthly_staff_wages: i32,
    pub staff: Vec<Staff>,
    pub brands: Vec<BrandDeal>,
    pub offers: Vec<SponsorOffer>,
    pub ledger: Vec<LedgerEntry>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct SaveMeta {
    pub path: String,
    pub file_name: String,
    pub game_date: String,
    pub season: i32,
    pub mod_name: String,
    pub game_version: String,
    pub user_team_id: i32,
    pub user_team: String,
    pub manager: String,
}

#[derive(Debug, Serialize)]
pub struct SaveData {
    pub meta: SaveMeta,
    pub cyclists: Vec<Cyclist>,
    pub teams: Vec<Team>,
    pub finance: Option<Finance>,
    /// Kept for older frontends: same as `meta.game_date`.
    pub game_date: String,
}

// ─── Small helpers ────────────────────────────────────────────────────────────

/// Column accessor that substitutes zeros/empties when a column is missing,
/// so older or modded databases still load.
struct Rows<'a> {
    db: &'a Cdb,
    table: &'static str,
    len: usize,
}

impl<'a> Rows<'a> {
    fn new(db: &'a Cdb, table: &'static str) -> Self {
        Rows { db, table, len: db.rows(table) }
    }
    fn int(&self, col: &str) -> Vec<i32> {
        self.db.ints(self.table, col).unwrap_or_else(|| vec![0; self.len])
    }
    fn float(&self, col: &str) -> Vec<f32> {
        self.db.floats(self.table, col).unwrap_or_else(|| vec![0.0; self.len])
    }
    fn text(&self, col: &str) -> Vec<String> {
        self.db.strings(self.table, col).unwrap_or_else(|| vec![String::new(); self.len])
    }
}

#[derive(Clone, Copy)]
struct Date {
    year: i32,
    month: i32,
    day: i32,
}

fn parse_date(v: i32) -> Option<Date> {
    if !(19000101..22000101).contains(&v) {
        return None;
    }
    let (year, month, day) = (v / 10000, (v % 10000) / 100, v % 100);
    ((1..=12).contains(&month) && (1..=31).contains(&day)).then_some(Date { year, month, day })
}

fn date_text(v: i32) -> String {
    parse_date(v)
        .map(|d| format!("{:04}-{:02}-{:02}", d.year, d.month, d.day))
        .unwrap_or_default()
}

fn age_at(birth: i32, now: Date) -> i32 {
    let Some(b) = parse_date(birth) else { return 0 };
    let mut age = now.year - b.year;
    if now.month < b.month || (now.month == b.month && now.day < b.day) {
        age -= 1;
    }
    if (0..100).contains(&age) { age } else { 0 }
}

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

fn hex_color(s: &str) -> String {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("#{}", s.to_uppercase())
    } else {
        String::new()
    }
}

// ─── Countries ────────────────────────────────────────────────────────────────

fn normalize_iso(raw: &str) -> String {
    let s: String = raw.chars().filter(|c| c.is_ascii_alphabetic()).take(3).collect::<String>().to_lowercase();
    match s.as_str() {
        "swd" => "swe".into(),
        "rom" => "rou".into(),
        "sar" => "rsa".into(),
        other => other.into(),
    }
}

/// ISO3 (PCM flavour) → (display name, ISO 3166 alpha-2).
fn country_info(iso: &str) -> (&'static str, &'static str) {
    match iso {
        "ago" => ("Angola", "ao"), "alb" => ("Albania", "al"), "and" => ("Andorra", "ad"),
        "arg" => ("Argentina", "ar"), "arm" => ("Armenia", "am"), "aus" => ("Australia", "au"),
        "aut" => ("Austria", "at"), "aze" => ("Azerbaijan", "az"), "bel" => ("Belgium", "be"),
        "ben" => ("Benin", "bj"), "ber" => ("Bermuda", "bm"), "bfa" => ("Burkina Faso", "bf"),
        "bhr" => ("Bahrain", "bh"), "bhs" => ("Bahamas", "bs"), "bih" => ("Bosnia and Herzegovina", "ba"),
        "blr" => ("Belarus", "by"), "blz" => ("Belize", "bz"), "bol" => ("Bolivia", "bo"),
        "bra" => ("Brazil", "br"), "brn" => ("Brunei", "bn"), "bul" => ("Bulgaria", "bg"),
        "can" => ("Canada", "ca"), "chi" => ("China", "cn"), "chn" => ("China", "cn"),
        "chl" => ("Chile", "cl"), "civ" => ("Ivory Coast", "ci"), "cmr" => ("Cameroon", "cm"),
        "cod" => ("DR Congo", "cd"), "col" => ("Colombia", "co"), "crc" => ("Costa Rica", "cr"),
        "cro" => ("Croatia", "hr"), "cub" => ("Cuba", "cu"), "cuw" => ("Curaçao", "cw"),
        "cyp" => ("Cyprus", "cy"), "cze" => ("Czechia", "cz"), "den" => ("Denmark", "dk"),
        "dom" => ("Dominican Republic", "do"), "dza" => ("Algeria", "dz"), "ecu" => ("Ecuador", "ec"),
        "egy" => ("Egypt", "eg"), "eri" => ("Eritrea", "er"), "esp" => ("Spain", "es"),
        "est" => ("Estonia", "ee"), "eth" => ("Ethiopia", "et"), "fin" => ("Finland", "fi"),
        "fra" => ("France", "fr"), "gab" => ("Gabon", "ga"), "gbr" => ("Great Britain", "gb"),
        "geo" => ("Georgia", "ge"), "ger" => ("Germany", "de"), "gha" => ("Ghana", "gh"),
        "grd" => ("Grenada", "gd"), "gre" => ("Greece", "gr"), "gtm" => ("Guatemala", "gt"),
        "gum" => ("Guam", "gu"), "guy" => ("Guyana", "gy"), "hkg" => ("Hong Kong", "hk"),
        "hnd" => ("Honduras", "hn"), "hun" => ("Hungary", "hu"), "idn" => ("Indonesia", "id"),
        "ind" => ("India", "in"), "irl" => ("Ireland", "ie"), "irn" => ("Iran", "ir"),
        "irq" => ("Iraq", "iq"), "isl" => ("Iceland", "is"), "isr" => ("Israel", "il"),
        "ita" => ("Italy", "it"), "jam" => ("Jamaica", "jm"), "jpn" => ("Japan", "jp"),
        "kaz" => ("Kazakhstan", "kz"), "ken" => ("Kenya", "ke"), "kgz" => ("Kyrgyzstan", "kg"),
        "khm" => ("Cambodia", "kh"), "kor" => ("South Korea", "kr"), "kos" => ("Kosovo", "xk"),
        "kuw" => ("Kuwait", "kw"), "lao" => ("Laos", "la"), "lat" => ("Latvia", "lv"),
        "lie" => ("Liechtenstein", "li"), "lka" => ("Sri Lanka", "lk"), "lso" => ("Lesotho", "ls"),
        "ltu" => ("Lithuania", "lt"), "lux" => ("Luxembourg", "lu"), "mar" => ("Morocco", "ma"),
        "mas" => ("Malaysia", "my"), "mco" => ("Monaco", "mc"), "mex" => ("Mexico", "mx"),
        "mkd" => ("North Macedonia", "mk"), "mli" => ("Mali", "ml"), "mlt" => ("Malta", "mt"),
        "mne" => ("Montenegro", "me"), "mng" => ("Mongolia", "mn"), "mol" => ("Moldova", "md"),
        "mus" => ("Mauritius", "mu"), "nam" => ("Namibia", "na"), "ned" => ("Netherlands", "nl"),
        "nga" => ("Nigeria", "ng"), "nor" => ("Norway", "no"), "nzl" => ("New Zealand", "nz"),
        "oma" => ("Oman", "om"), "pak" => ("Pakistan", "pk"), "pan" => ("Panama", "pa"),
        "per" => ("Peru", "pe"), "phl" => ("Philippines", "ph"), "pol" => ("Poland", "pl"),
        "por" => ("Portugal", "pt"), "pri" => ("Puerto Rico", "pr"), "pry" => ("Paraguay", "py"),
        "pse" => ("Palestine", "ps"), "qat" => ("Qatar", "qa"), "rou" => ("Romania", "ro"),
        "rsa" => ("South Africa", "za"), "rus" => ("Russia", "ru"), "rwa" => ("Rwanda", "rw"),
        "sau" => ("Saudi Arabia", "sa"), "sen" => ("Senegal", "sn"), "ser" => ("Serbia", "rs"),
        "sgp" => ("Singapore", "sg"), "slo" => ("Slovenia", "si"), "smr" => ("San Marino", "sm"),
        "svk" => ("Slovakia", "sk"), "swe" => ("Sweden", "se"), "swi" => ("Switzerland", "ch"),
        "syr" => ("Syria", "sy"), "tha" => ("Thailand", "th"), "tls" => ("Timor-Leste", "tl"),
        "tto" => ("Trinidad and Tobago", "tt"), "tun" => ("Tunisia", "tn"), "tur" => ("Türkiye", "tr"),
        "twn" => ("Taiwan", "tw"), "uae" => ("United Arab Emirates", "ae"), "uga" => ("Uganda", "ug"),
        "ukr" => ("Ukraine", "ua"), "uru" => ("Uruguay", "uy"), "usa" => ("United States", "us"),
        "uzb" => ("Uzbekistan", "uz"), "ven" => ("Venezuela", "ve"), "vnm" => ("Vietnam", "vn"),
        "zim" => ("Zimbabwe", "zw"),
        _ => ("", ""),
    }
}

fn continent_name(constant: &str) -> &'static str {
    match constant {
        "Europa" | "Europe" => "Europe",
        "NorthAmerica" => "North America",
        "SouthAmerica" => "South America",
        "Asia" => "Asia",
        "Oceania" => "Oceania",
        "Africa" => "Africa",
        _ => "Unknown",
    }
}

#[derive(Clone, Default)]
struct Country {
    iso: String,
    name: String,
    flag: String,
    continent: String,
}

fn load_countries(db: &Cdb) -> HashMap<i32, Country> {
    let continents: HashMap<i32, &'static str> = {
        let t = Rows::new(db, "STA_continent");
        t.int("IDcontinent").into_iter().zip(t.text("CONSTANT")).map(|(id, c)| (id, continent_name(&c))).collect()
    };
    let t = Rows::new(db, "STA_country");
    let (ids, codes, conts) = (t.int("IDcountry"), t.text("CONSTANT"), t.int("fkIDcontinent"));
    (0..t.len)
        .map(|i| {
            let iso = normalize_iso(&codes[i]);
            let (name, flag) = country_info(&iso);
            let country = Country {
                name: if name.is_empty() { iso.to_uppercase() } else { name.to_string() },
                flag: flag.to_string(),
                continent: continents.get(&conts[i]).copied().unwrap_or("Unknown").to_string(),
                iso,
            };
            (ids[i], country)
        })
        .collect()
}

// ─── Rider metrics ────────────────────────────────────────────────────────────

const STATS: [(&str, &str); 14] = [
    ("flat", "Flat"), ("mountain", "Mountain"), ("med_mtn", "Med. Mountain"), ("downhill", "Downhill"),
    ("cobble", "Cobbles"), ("timetrial", "Time Trial"), ("prologue", "Prologue"), ("sprint", "Sprint"),
    ("acceleration", "Acceleration"), ("endurance", "Endurance"), ("resistance", "Resistance"),
    ("recuperation", "Recuperation"), ("hill", "Hill"), ("baroudeur", "Baroudeur"),
];

fn stat_pairs(c: &Cyclist) -> [(i32, i32); 14] {
    [
        (c.flat, c.flat_p), (c.mountain, c.mountain_p), (c.med_mtn, c.med_mtn_p), (c.downhill, c.downhill_p),
        (c.cobble, c.cobble_p), (c.timetrial, c.timetrial_p), (c.prologue, c.prologue_p), (c.sprint, c.sprint_p),
        (c.acceleration, c.acceleration_p), (c.endurance, c.endurance_p), (c.resistance, c.resistance_p),
        (c.recuperation, c.recuperation_p), (c.hill, c.hill_p), (c.baroudeur, c.baroudeur_p),
    ]
}

fn derive_metrics(c: &mut Cyclist) {
    let pairs = stat_pairs(c);
    let n = pairs.len() as f32;
    let avg = pairs.iter().map(|p| p.0 as f32).sum::<f32>() / n;
    let ceil = pairs.iter().map(|p| p.1.max(p.0) as f32).sum::<f32>() / n;
    c.skill_average = round1(avg);
    c.skill_ceiling = round1(ceil);
    c.growth = round1((ceil - avg).max(0.0));
    c.peak_gap = pairs.iter().map(|p| (p.1 - p.0).max(0)).max().unwrap_or(0);
    let mut ranked: Vec<TopSkill> = STATS
        .iter()
        .zip(pairs.iter())
        .map(|((k, l), p)| TopSkill { key: k.to_string(), label: l.to_string(), value: p.0 })
        .collect();
    ranked.sort_by(|a, b| b.value.cmp(&a.value));
    ranked.truncate(3);
    c.specialty_rating = ranked.iter().map(|s| s.value).sum();
    c.top_skills = ranked;
    c.scout_grade = scout_grade(c).to_string();
}

fn scout_grade(c: &Cyclist) -> &'static str {
    let (age, stars, upside, ca) = (c.age, c.potential, c.growth, c.current_ability);
    if age > 0 && age <= 20 && stars >= 5.5 && upside >= 6.0 { return "Wonderkid"; }
    if age > 0 && age <= 22 && stars >= 5.0 && upside >= 4.5 { return "Elite Prospect"; }
    if age > 0 && age <= 23 && upside >= 5.0 && ca < 72.0 { return "Late Bloomer"; }
    if age > 0 && age <= 23 && ca >= 72.0 { return "Ready Now"; }
    if age >= 30 && upside >= 3.0 { return "Past Peak"; }
    if age >= 27 && ca >= 78.0 { return "Veteran"; }
    "Monitor"
}

fn rider_type(id: i32) -> &'static str {
    match id {
        1 => "Stage Racer",
        2 => "Climber",
        3 => "Time Trialist",
        4 => "Sprinter",
        5 => "Puncheur",
        6 => "Cobbles",
        7 => "Rouleur",
        _ => "Unknown",
    }
}

fn ledger_kind(code: i32) -> (&'static str, &'static str) {
    match code {
        3011 => ("balance", "Balance carried forward"),
        5484 => ("sponsor", "Sponsor budget"),
        2918 => ("wages", "Rider wages"),
        2919 => ("staff", "Staff wages"),
        3072 => ("equipment", "Equipment"),
        330 => ("training", "Training camp"),
        4124 => ("staff", "Staff contract"),
        4141 => ("prize", "Prize money"),
        _ => ("other", "Other"),
    }
}

// ─── Extraction ───────────────────────────────────────────────────────────────

struct Contract {
    wage: i32,
    start: i32,
    end: i32,
}

pub fn extract(db: &Cdb, path: &str) -> Result<SaveData, String> {
    if db.table("DYN_cyclist").is_none() || db.table("DYN_team").is_none() {
        return Err("This file has no rider or team tables — is it a career save?".into());
    }

    // Game date & user
    let cfg = Rows::new(db, "GAM_config");
    let date_int = cfg.int("gene_i_date").first().copied().unwrap_or(0);
    let now = parse_date(date_int).unwrap_or(Date { year: 2026, month: 1, day: 1 });
    let users = Rows::new(db, "GAM_user");
    let (active, user_teams, user_names) = (users.int("game_i_active"), users.int("fkIDteam_duplicate"), users.text("game_sz_display_name"));
    let user_idx = (0..users.len).find(|&i| active[i] == 1 && user_teams[i] > 0);
    let user_team_id = user_idx.map(|i| user_teams[i]).unwrap_or(0);

    let countries = load_countries(db);
    let region_country: HashMap<i32, i32> = {
        let t = Rows::new(db, "STA_region");
        t.int("IDregion").into_iter().zip(t.int("fkIDcountry")).collect()
    };
    let divisions: HashMap<i32, (String, i32)> = {
        let t = Rows::new(db, "STA_division");
        t.int("IDdivision")
            .into_iter()
            .zip(t.text("CONSTANT"))
            .map(|(id, c)| {
                // "[1] World Tour" → ("World Tour", 1)
                let tier = c.strip_prefix('[').and_then(|r| r.split(']').next()).and_then(|n| n.trim().parse().ok()).unwrap_or(0);
                let name = c.split(']').last().unwrap_or(&c).trim().to_string();
                (id, (name, tier))
            })
            .collect()
    };

    // Contracts
    let contracts: HashMap<i32, Contract> = {
        let t = Rows::new(db, "DYN_contract_cyclist");
        let (ids, wages, starts, ends) = (t.int("IDcontract_cyclist"), t.int("finan_i_period_wage"), t.int("iYearBegin"), t.int("iYearEnd"));
        (0..t.len).map(|i| (ids[i], Contract { wage: wages[i], start: starts[i], end: ends[i] })).collect()
    };

    // Staff payroll per team
    let mut staff_payroll: HashMap<i32, i32> = HashMap::new();
    let mut my_staff = Vec::new();
    for (table, id_col, role) in [("DYN_coach", "IDcoach", "Coach"), ("DYN_physician", "IDphysician", "Physician"), ("DYN_scout", "IDscout", "Scout")] {
        let t = Rows::new(db, table);
        let (ids, teams, wages, ends, first, last) = (t.int(id_col), t.int("fkIDteam"), t.int("finan_i_wage"), t.int("gene_i_contract_end"), t.text("gene_sz_firstname"), t.text("gene_sz_lastname"));
        for i in 0..t.len {
            if teams[i] <= 0 {
                continue;
            }
            *staff_payroll.entry(teams[i]).or_default() += wages[i];
            if teams[i] == user_team_id {
                my_staff.push(Staff {
                    id: ids[i],
                    role: role.into(),
                    name: format!("{} {}", first[i], last[i]).trim().to_string(),
                    wage: wages[i],
                    contract_end: ends[i],
                });
            }
        }
    }

    // Sponsors
    let sponsor_names: HashMap<i32, String> = {
        let t = Rows::new(db, "DYN_sponsor");
        t.int("IDsponsor").into_iter().zip(t.text("gene_sz_name")).collect()
    };
    struct SponsorDeal { row_id: i32, id: i32, start: i32, end: i32, budget: i32, next: i32 }
    let team_sponsor: HashMap<i32, SponsorDeal> = {
        let t = Rows::new(db, "DYN_team_sponsor");
        let (row_ids, teams, sp, st, en, bu, nx) = (t.int("IDteam_sponsor"), t.int("fkIDteam"), t.int("fkIDsponsor"), t.int("value_i_contract_year_start"), t.int("value_i_contract_year_end"), t.int("value_i_budget"), t.int("value_i_budget_next"));
        let mut m: HashMap<i32, SponsorDeal> = HashMap::new();
        for i in 0..t.len {
            // Prefer the deal covering the current season, else the latest one.
            let deal = SponsorDeal { row_id: row_ids[i], id: sp[i], start: st[i], end: en[i], budget: bu[i], next: nx[i] };
            let covers = deal.start <= now.year && now.year <= deal.end;
            match m.get(&teams[i]) {
                Some(prev) if (prev.start <= now.year && now.year <= prev.end) && !covers => {}
                Some(prev) if !covers && prev.end >= deal.end => {}
                _ => { m.insert(teams[i], deal); }
            }
        }
        m
    };

    // Teams
    let tt = Rows::new(db, "DYN_team");
    let (t_ids, t_names, t_shorts, t_abbr, t_countries, t_div, t_budget, t_eval, t_c1, t_c2) = (
        tt.int("IDteam"), tt.text("gene_sz_name"), tt.text("gene_sz_shortname"), tt.text("abbreviation"),
        tt.int("fkIDcountry"), tt.int("fkIDdivision"), tt.int("value_i_budget"), tt.float("value_f_current_evaluation"),
        tt.text("gene_sz_color"), tt.text("gene_sz_secondary_color"),
    );
    let mut teams: Vec<Team> = Vec::new();
    for i in 0..tt.len {
        let id = t_ids[i];
        if id <= 0 || id == FREE_AGENT_TEAM_ID || t_names[i].is_empty() || t_names[i] == "-" {
            continue;
        }
        let country = countries.get(&t_countries[i]).cloned().unwrap_or_default();
        let (division, tier) = divisions.get(&t_div[i]).cloned().unwrap_or_default();
        let deal = team_sponsor.get(&id);
        teams.push(Team {
            id,
            name: t_names[i].clone(),
            short: if t_shorts[i].is_empty() { t_names[i].clone() } else { t_shorts[i].clone() },
            abbreviation: t_abbr[i].clone(),
            country_iso: country.iso.clone(),
            country_name: country.name.clone(),
            flag: country.flag.clone(),
            color1: hex_color(&t_c1[i]),
            color2: hex_color(&t_c2[i]),
            division_id: t_div[i],
            division,
            tier,
            budget: t_budget[i],
            sponsor_id: deal.map(|d| d.id).unwrap_or(0),
            sponsor_deal_id: deal.map(|d| d.row_id).unwrap_or(0),
            sponsor: deal.and_then(|d| sponsor_names.get(&d.id).cloned()).unwrap_or_default(),
            sponsor_budget: deal.map(|d| d.budget).unwrap_or(0),
            sponsor_budget_next: deal.map(|d| d.next).unwrap_or(0),
            sponsor_contract_end: deal.map(|d| d.end).unwrap_or(0),
            staff_payroll: staff_payroll.get(&id).copied().unwrap_or(0),
            evaluation: round1(t_eval[i]),
            is_mine: id == user_team_id,
            ..Default::default()
        });
    }
    let team_index: HashMap<i32, usize> = teams.iter().enumerate().map(|(i, t)| (t.id, i)).collect();

    // Market & scouting
    let market: HashSet<i32> = db.ints("DYN_transfer_available_cyclist", "IDtransfer_available_cyclist").unwrap_or_default().into_iter().collect();
    let my_scouts: HashSet<i32> = {
        let t = Rows::new(db, "DYN_scout");
        t.int("IDscout").into_iter().zip(t.int("fkIDteam")).filter(|&(_, team)| team == user_team_id && team > 0).map(|(id, _)| id).collect()
    };
    struct Report { date: i32, mine: bool, pots: [f32; 7] }
    let mut reports: HashMap<i32, (i32, Report)> = HashMap::new(); // cyclist → (count, best report)
    {
        let t = Rows::new(db, "DYN_scout_report");
        let (cyc, scout, date) = (t.int("fkIDcyclist"), t.int("fkIDscout"), t.int("gene_date_report"));
        let pots: Vec<Vec<f32>> = [
            "value_f_potential_tr_tour", "value_f_potential_tr_mountain", "value_f_potential_tr_timetrial",
            "value_f_potential_tr_sprint", "value_f_potential_tr_ardenaises", "value_f_potential_tr_flandriennes",
            "value_f_potential_tr_flat",
        ]
        .iter()
        .map(|c| t.float(c))
        .collect();
        for i in 0..t.len {
            let rep = Report { date: date[i], mine: my_scouts.contains(&scout[i]), pots: std::array::from_fn(|k| pots[k][i]) };
            let entry = reports.entry(cyc[i]).or_insert((0, Report { date: 0, mine: false, pots: [0.0; 7] }));
            entry.0 += 1;
            // Prefer your own scouts, then the most recent report.
            let better = (rep.mine && !entry.1.mine) || (rep.mine == entry.1.mine && rep.date >= entry.1.date);
            if better {
                entry.1 = rep;
            }
        }
    }

    // Riders
    let ct = Rows::new(db, "DYN_cyclist");
    let ids = ct.int("IDcyclist");
    let (first, last, full) = (ct.text("gene_sz_firstname"), ct.text("gene_sz_lastname"), ct.text("gene_sz_firstlastname"));
    let (team_ref, contract_ref, region, birth, type_id) = (ct.int("fkIDteam"), ct.int("fkIDcontract"), ct.int("fkIDregion"), ct.int("gene_i_birthdate"), ct.int("fkIDtype_rider"));
    let (size, weight, state, retire, wins) = (ct.int("gene_i_size"), ct.int("gene_i_weight"), ct.int("fkIDcyclist_state"), ct.int("gene_b_will_retire"), ct.int("gene_i_nb_total_victory"));
    let (ability, potential, popularity) = (ct.float("value_f_current_ability"), ct.float("value_f_potentiel"), ct.float("gene_f_popularity"));
    let (tour, classic) = (ct.int("charac_i_tour"), ct.int("charac_i_classic"));
    let stat = |c: &str| ct.int(c);
    let s: Vec<(Vec<i32>, Vec<i32>)> = [
        "plain", "mountain", "medium_mountain", "downhilling", "cobble", "timetrial", "prologue", "sprint",
        "acceleration", "endurance", "resistance", "recuperation", "hill", "baroudeur",
    ]
    .iter()
    .map(|k| (stat(&format!("charac_i_{k}")), stat(&format!("limit_i_{k}"))))
    .collect();
    let clamp = |v: i32| v.clamp(0, 100);

    let mut cyclists = Vec::with_capacity(ct.len);
    for i in 0..ct.len {
        let id = ids[i];
        if id <= 0 {
            continue;
        }
        let team_id = team_ref[i];
        let free_agent = team_id == FREE_AGENT_TEAM_ID || team_id <= 0;
        let team = team_index.get(&team_id).map(|&ti| &teams[ti]);
        let (team_name, team_short, division) = match team {
            Some(t) => (t.name.clone(), t.short.clone(), t.division.clone()),
            None if free_agent => ("Free Agent".into(), "Free Agent".into(), String::new()),
            None => (format!("Team #{team_id}"), format!("#{team_id}"), String::new()),
        };
        let country = region_country.get(&region[i]).and_then(|cid| countries.get(cid)).cloned().unwrap_or_default();
        let contract = contracts.get(&contract_ref[i]).filter(|_| !free_agent);
        let name = {
            let combined = format!("{} {}", first[i], last[i]).trim().to_string();
            if !combined.is_empty() { combined } else if !full[i].is_empty() { full[i].clone() } else { format!("#{id}") }
        };
        let mut c = Cyclist {
            id,
            name,
            firstname: first[i].clone(),
            lastname: last[i].clone(),
            team_id,
            team: team_name,
            team_short,
            division,
            nationality: if country.name.is_empty() { "Unknown".into() } else { country.name.clone() },
            continent: if country.continent.is_empty() { "Unknown".into() } else { country.continent.clone() },
            iso: country.iso.clone(),
            flag: country.flag.clone(),
            birthdate: birth[i].to_string(),
            age: age_at(birth[i], now),
            signable: parse_date(birth[i]).map(|b| now.year - b.year >= 18).unwrap_or(true),
            signable_from: parse_date(birth[i]).map(|b| b.year + 18).unwrap_or(0),
            rider_type: rider_type(type_id[i]).into(),
            rider_type_id: type_id[i],
            size: size[i],
            weight: weight[i],
            current_ability: round1(ability[i]),
            potential: (potential[i] * 100.0).round() / 100.0,
            free_agent,
            is_mine: team_id == user_team_id && user_team_id > 0,
            wage: contract.map(|c| c.wage).unwrap_or(0),
            contract_start: contract.map(|c| c.start).unwrap_or(0),
            contract_end: contract.map(|c| c.end).unwrap_or(0),
            popularity: round1(popularity[i]),
            wins: wins[i],
            tour_rating: tour[i],
            classic_rating: classic[i],
            injured: matches!(state[i], 2 | 4),
            will_retire: retire[i] != 0,
            on_market: market.contains(&id),
            ..Default::default()
        };
        let v = |k: usize| (clamp(s[k].0[i]), clamp(s[k].1[i]));
        (c.flat, c.flat_p) = v(0);
        (c.mountain, c.mountain_p) = v(1);
        (c.med_mtn, c.med_mtn_p) = v(2);
        (c.downhill, c.downhill_p) = v(3);
        (c.cobble, c.cobble_p) = v(4);
        (c.timetrial, c.timetrial_p) = v(5);
        (c.prologue, c.prologue_p) = v(6);
        (c.sprint, c.sprint_p) = v(7);
        (c.acceleration, c.acceleration_p) = v(8);
        (c.endurance, c.endurance_p) = v(9);
        (c.resistance, c.resistance_p) = v(10);
        (c.recuperation, c.recuperation_p) = v(11);
        (c.hill, c.hill_p) = v(12);
        (c.baroudeur, c.baroudeur_p) = v(13);
        derive_metrics(&mut c);

        if let Some((count, rep)) = reports.get(&id) {
            c.scout_reports = *count;
            c.my_report = rep.mine;
            c.scout_report_date = date_text(rep.date);
            [c.scout_tour_potential, c.scout_mountain_potential, c.scout_timetrial_potential, c.scout_sprint_potential,
             c.scout_ardennes_potential, c.scout_cobble_potential, c.scout_flat_potential] = rep.pots;
            c.scout_estimate = rep.pots.iter().copied().fold(0.0, f32::max);
        }

        if let Some(&ti) = team_index.get(&team_id) {
            let t = &mut teams[ti];
            t.riders += 1;
            t.payroll += c.wage;
            t.avg_ca += c.current_ability;
            t.top_ca = t.top_ca.max(c.current_ability);
        }
        cyclists.push(c);
    }
    for t in &mut teams {
        if t.riders > 0 {
            t.avg_ca = round1(t.avg_ca / t.riders as f32);
        }
    }
    cyclists.sort_by(|a, b| b.current_ability.total_cmp(&a.current_ability));
    teams.sort_by(|a, b| a.name.cmp(&b.name));

    let finance = (user_team_id > 0).then(|| build_finance(db, user_team_id, &teams, &sponsor_names, my_staff));

    let file_name = std::path::Path::new(path).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
    let user_team = teams.iter().find(|t| t.is_mine).map(|t| t.name.clone()).unwrap_or_default();
    let game_date = date_text(date_int);
    Ok(SaveData {
        meta: SaveMeta {
            path: path.to_string(),
            file_name,
            game_date: game_date.clone(),
            season: now.year,
            mod_name: cfg.text("gene_sz_modname").first().cloned().unwrap_or_default(),
            game_version: cfg.text("game_sz_version").first().cloned().unwrap_or_default(),
            user_team_id,
            user_team,
            manager: user_idx.map(|i| user_names[i].clone()).unwrap_or_default(),
        },
        cyclists,
        teams,
        finance,
        game_date,
    })
}

fn build_finance(db: &Cdb, team_id: i32, teams: &[Team], sponsor_names: &HashMap<i32, String>, mut staff: Vec<Staff>) -> Finance {
    let team = teams.iter().find(|t| t.id == team_id).cloned().unwrap_or_default();
    let balance = career_balance(db);

    let brands: Vec<BrandDeal> = {
        let names: HashMap<i32, (String, i32)> = {
            let t = Rows::new(db, "STA_brand");
            let (ids, n, ty) = (t.int("IDbrand"), t.text("gene_sz_name"), t.int("fkIDbrand_type"));
            (0..t.len).map(|i| (ids[i], (n[i].clone(), ty[i]))).collect()
        };
        let types: HashMap<i32, String> = {
            let t = Rows::new(db, "STA_brand_type");
            t.int("IDbrand_type").into_iter().zip(t.text("CONSTANT")).collect()
        };
        let t = Rows::new(db, "DYN_brand_contract");
        let (ids, brand, year, budget, left, teams_col) = (t.int("IDbrand_contract"), t.int("fkIDbrand"), t.int("gene_i_year_contract"), t.int("value_i_budget"), t.int("value_i_years_left"), t.int("fkIDteam"));
        let mut v: Vec<BrandDeal> = (0..t.len)
            .filter(|&i| teams_col[i] == team_id && (left[i] > 0 || budget[i] > 0))
            .map(|i| {
                let (name, ty) = names.get(&brand[i]).cloned().unwrap_or_default();
                let category = types.get(&ty).map(|c| {
                    let mut s = c.to_lowercase();
                    if let Some(f) = s.get_mut(0..1) { f.make_ascii_uppercase(); }
                    s
                }).unwrap_or_default();
                BrandDeal { id: ids[i], brand: name, category, budget: budget[i], year: year[i], years_left: left[i] }
            })
            .collect();
        v.sort_by(|a, b| b.budget.cmp(&a.budget));
        v
    };

    let offers: Vec<SponsorOffer> = {
        let t = Rows::new(db, "DYN_sponsor_offer");
        let (ids, contact, deadline, budget, state, duration) = (t.int("IDsponsor_offer"), t.int("value_i_contact_date"), t.int("value_i_deadline_date"), t.int("value_i_budget"), t.int("value_i_state"), t.int("value_i_duration"));
        (0..t.len)
            .map(|i| SponsorOffer {
                sponsor_id: ids[i],
                sponsor: sponsor_names.get(&ids[i]).cloned().unwrap_or_else(|| format!("Sponsor #{}", ids[i])),
                budget: budget[i],
                duration: duration[i],
                contact_date: date_text(contact[i]),
                deadline: date_text(deadline[i]),
                state: state[i],
            })
            .collect()
    };

    let ledger: Vec<LedgerEntry> = {
        let t = Rows::new(db, "DYN_finance");
        let (ids, codes, dates, values, args) = (t.int("IDoperation"), t.int("gene_strID_operation_string"), t.int("gene_i_date"), t.int("gene_i_value"), t.text("gene_sz_argument"));
        let mut v: Vec<(i32, LedgerEntry)> = (0..t.len)
            .map(|i| {
                let (category, label) = ledger_kind(codes[i]);
                (dates[i], LedgerEntry {
                    id: ids[i],
                    date: date_text(dates[i]),
                    amount: values[i],
                    category: category.into(),
                    label: if category == "other" { format!("Other (#{})", codes[i]) } else { label.into() },
                    detail: args[i].clone(),
                })
            })
            .collect();
        // IDs restart every season, so order by date first and keep file order within a day.
        v.sort_by_key(|(d, _)| *d);
        v.into_iter().map(|(_, e)| e).collect()
    };

    staff.sort_by(|a, b| a.role.cmp(&b.role).then(b.wage.cmp(&a.wage)));
    Finance {
        team_id,
        balance: balance.map(|b| b as f64).unwrap_or(0.0),
        balance_editable: balance.is_some(),
        season_budget: team.budget,
        sponsor_id: team.sponsor_id,
        sponsor_deal_id: team.sponsor_deal_id,
        sponsor: team.sponsor.clone(),
        sponsor_budget: team.sponsor_budget,
        sponsor_budget_next: team.sponsor_budget_next,
        sponsor_contract_start: {
            let t = Rows::new(db, "DYN_team_sponsor");
            let (teams_col, sp, st) = (t.int("fkIDteam"), t.int("fkIDsponsor"), t.int("value_i_contract_year_start"));
            (0..t.len).find(|&i| teams_col[i] == team_id && sp[i] == team.sponsor_id).map(|i| st[i]).unwrap_or(0)
        },
        sponsor_contract_end: team.sponsor_contract_end,
        monthly_rider_wages: team.payroll,
        monthly_staff_wages: team.staff_payroll,
        staff,
        brands,
        offers,
        ledger,
    }
}

/// Cash balance: the `SOLDE` row of `GAM_career_data`.
pub fn career_balance(db: &Cdb) -> Option<f32> {
    let row = db.find_row_str("GAM_career_data", "CONSTANT", "SOLDE")?;
    db.floats("GAM_career_data", "value")?.get(row).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_career_save() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
        for name in ["Career_2 copy.cdb", "Career_2.cdb"] {
            let path = format!("{root}/{name}");
            if !std::path::Path::new(&path).is_file() {
                continue;
            }
            let db = Cdb::open(&path).unwrap();
            let data = extract(&db, &path).unwrap();
            assert_eq!(data.cyclists.len(), db.rows("DYN_cyclist"));
            assert!(data.teams.len() > 100);
            assert!(data.meta.user_team_id > 0);
            let fin = data.finance.as_ref().unwrap();
            assert!(fin.balance_editable);
            assert!(fin.monthly_rider_wages > 0);
            let mine: Vec<_> = data.cyclists.iter().filter(|c| c.is_mine).collect();
            assert!(!mine.is_empty());
            assert_eq!(mine.iter().map(|c| c.wage).sum::<i32>(), fin.monthly_rider_wages);
            let named = data.cyclists.iter().filter(|c| !c.name.starts_with('#')).count();
            assert!(named * 100 / data.cyclists.len() > 98, "{name}: too many unnamed riders");
            let known = data.cyclists.iter().filter(|c| c.nationality != "Unknown").count();
            assert!(known * 100 / data.cyclists.len() > 95, "{name}: too many unknown countries");
            println!(
                "{name}: {} riders, {} teams, team={} balance={} wages={} staff={} prospects={}",
                data.cyclists.len(), data.teams.len(), data.meta.user_team, fin.balance,
                fin.monthly_rider_wages, fin.monthly_staff_wages,
                data.cyclists.iter().filter(|c| c.scout_reports > 0).count()
            );
        }
    }
}

#[cfg(test)]
mod dump {
    /// `PCM_DUMP=<out.json> cargo test dump_json -- --ignored` writes the UI model for browser previews.
    #[test]
    #[ignore]
    fn dump_json() {
        let out = std::env::var("PCM_DUMP").expect("set PCM_DUMP");
        let src = std::env::var("PCM_SAVE").unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../Career_2 copy.cdb").into());
        let db = crate::cdb::Cdb::open(&src).unwrap();
        let data = super::extract(&db, &src).unwrap();
        std::fs::write(out, serde_json::to_string(&data).unwrap()).unwrap();
    }
}
