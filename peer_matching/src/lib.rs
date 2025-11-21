use std::vec;
use std::iter::zip;
use std::collections::HashSet;

/// topics
///
/// for classifying posts
/// for identifying matches
#[derive(Clone)]
pub enum Topics {
    Housing,
    VisaAdmin,
    Coursework,
    StudyHelp,
    SocialBelonging,
    MentalHealth,
    CampusLogistics,
    MoneyWork,
    LanguageSupport,
}

// this is really just an array
// but for lin alg calling vector
pub struct TopicVector([bool; 9]);
// common operations
impl TopicVector {
    pub fn new() -> Self {
        Self([false; 9])
    }

    // only ever set from false -> true on 1 query
    fn set(&mut self, topic: Topics) {
        // set arr idx == topic to 0
        self.0[topic as usize] = true; 
    }
}


/// enum for country of origin -- only set values are allowed
/// ensure this in the frontend + datastore
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Country {
    China,
    India,
    SouthKorea,
    Vietnam,
    Japan,
    Nigeria,
    Ghana,
    Kenya,
    Mexico,
    Brazil,
    Canada,
    UnitedKingdom,
    Germany,
    France,
    SaudiArabia,
    Turkey,
    Iran,
    Nepal,
    Bangladesh,
    Pakistan,
    Other,   // catch-all fallback
}

pub enum UserRole {
    Peer,
    Mentor,
    Mentee,
}

/// Define Structs -> UserInfo = static
/// Post -> Ref User, sometimes changes
/// UserState -> Derived topics/Needs/Experience Vectors
pub struct UserInfo {
    pub did: String,
    pub display_name: String,

    pub country: Country,
    pub study_subjects: Vec<String>,  // todo create Enums
    pub hobbies: Vec<String>,   // todo create hobby categories Enum
    pub languages: Vec<String>, // todo Create Language  Enum

    pub arrival_date: String, // or datetime
    pub role: UserRole,
}

/// PostKind for classifying posts-> mentor experiene
/// or Need vector for user
#[derive(Eq, PartialEq, Clone)]
pub enum PostKind {
    HelpAsk,
    HelpReply,
    Other,
}


/// author, kind, text, topics
#[derive(Clone)]
pub struct Post {
    pub author_id: String,  // link to User DiD
    pub kind: PostKind,
    pub text: String,
    pub flagged: bool,      // flag for toxic content

    // populated by classifier
    pub topics: Vec<Topics>,
}

/// classify topics in a blob of text
/// current implementation takes a blob of text
/// filters for certain words associated per topic
///
fn classify_topics(text: &str) -> Vec<Topics> {
    let mut result: Vec<Topics> = vec![];
    let lower = text.to_lowercase();
    //TODO improvement - variable hit count for topic matching
    // longer the text... require more topic mentions

    // count keyword hits for each topic
    // select topics that meet threshold
    // use topic keyword dispatch table iterator
    for (topic, keywords) in TOPIC_KEYWORD_TABLE.iter() {
        // iterator with filter of contains (note subset matching)
        // not word matching... maybe issues but ignore for now
        let hits = keywords
                    .iter()
                    .filter(|kw| lower.contains(*kw))
                    .count();
        if hits >= 1 {
            result.push(topic.clone());
        }
    }
    result
}

/// Check all posts for a given user and define need params
pub fn compute_need_vector(_posts: Vec<Post>) -> TopicVector {
    let mut need_vector: Vec<Topics> = vec![];
    // select Posts where kind == HelpAsk
    let post_ask = _posts.iter()
                    .filter(|p| p.kind == PostKind::HelpAsk);
    for p in post_ask {
        let topics = classify_topics(&p.text);
        need_vector.extend(topics);
    }

    return build_topic_vector(need_vector);
}

pub fn compute_experience_vector(_posts: Vec<Post>) -> TopicVector {
    let mut exp_vector: Vec<Topics> = vec![];
    // select Posts where kind == HelpAsk
    let post_ask = _posts.iter()
                    .filter(|p| p.kind == PostKind::HelpReply);
    for p in post_ask {
        let topics = classify_topics(&p.text);
        exp_vector.extend(topics);
    }

    return build_topic_vector(exp_vector);
}

fn build_topic_vector(in_topics: Vec<Topics>) -> TopicVector {
    let mut new_topic_vector = TopicVector::new();

    // set
    for t in in_topics {
        // set idx pos of topic to true
        new_topic_vector.set(t);
    }
    new_topic_vector
}


/// topic similarity
/// for now calculate distance away from user_topic = 1 and mentor = 0
/// do not penalize for user_topic = 0 and mentor = 1
/// outputs u8 that has amount topics mentor doesnt have
fn topic_similarity(user_topic_vector: TopicVector, mentor_topic_vector: TopicVector) -> f64 {
    // count where user and mentor share a topic
    let matches = zip(user_topic_vector.0.iter(), mentor_topic_vector.0.iter())
        .filter(|&(&user_topic_vector, &mentor_topic_vector)| user_topic_vector && mentor_topic_vector)
        .count() as f64;
    
    let user_topic_count = user_topic_vector.0
        .iter()
        .filter(|&user_topic_vector| *user_topic_vector)
        .count() as f64;
    // value = match - penalty -- improve by normalizing values to only Mentee Needs
    let result = matches / user_topic_count;

    return result;
}

fn origin_match(_user_origin: Country, _mentor_origin: Country) -> f64{
    (_user_origin == _mentor_origin) as i32 as f64
}

/// interest/hobby overlap
/// compare as strings
/// not penaliznig for differences
/// rewarding for similarities
///
const E: f64 = 2.71828f64;
fn hobby_overlap(user_hobby: Vec<&str>, mentor_hobby: Vec<&str>) -> f64 {
    let mut hits: usize = 0;
    for hobby in user_hobby {
        if mentor_hobby.contains(&hobby) {
            hits += 1
        }
    }
    let hits = hits as f64;
    // choose params for max --- max = 5 for now.. anything above goes down
    // sigmoid params e^-1.5(x - 3) -> looks decent.
    // y = 0...1   x = num of matches
    let sigmoid: f64 = 1f64 / (1f64 + E.powf(-1.5 * (hits - 3.0)));
    return sigmoid
}

// Jaccard distance using HashSets
// returns 0 - 1 1 representing 
// exact same
// 0 represent no shared at all
fn jaccard_similarity(a: &[&str], b: &[&str]) -> f64 {
    let set_a: HashSet<_> = a.iter().cloned().collect();
    let set_b: HashSet<_> = b.iter().cloned().collect();

    // get intersect and union
    let intersect_size = set_a.intersection(&set_b).count();
    let union_size = set_a.union(&set_b).count();

    if union_size == 0 {
        0.0
    } else {
        intersect_size as f64 / union_size as f64
    }
}


fn subject_similarity(user_subjects: Vec<&str>, mentor_subjects: Vec<&str>) -> f64 {
   jaccard_similarity(&user_subjects, &mentor_subjects)
}


/// score for any user, mentor pairing -> mentors must have mentor tag
///     OR meet a minimum number of qualifications.
/// Calculates score based on 
/// topic match - bin arr similarity
/// origin match -> yes/no
/// interest_overlap - Jaccard
/// study Subjusts - Jaccard 
/// language score -> 1 = match 1 languge, 0 else
/// helpful_score -> ratio of endorsements/posts
pub fn mentor_match_score(user_info: &UserInfo, mentor_info: &UserInfo, user_posts: &[Post], mentor_posts: &[Post],) -> f64 {
    // close to final implementation
    // need user, mentor topic vec
    // need origin, hobby, subjects for user,mentor
    let user_origin = user_info.country;
    let mentor_origin = mentor_info.country;
    let user_hobbies: Vec<&str> = user_info.hobbies.iter().map(|s| s.as_str()).collect();
    let mentor_hobbies: Vec<&str> = mentor_info.hobbies.iter().map(|s| s.as_str()).collect();
    let user_subjects: Vec<&str> = user_info
        .study_subjects
        .iter()
        .map(|s| s.as_str())
        .collect();
    let mentor_subjects: Vec<&str> = mentor_info
        .study_subjects
        .iter()
        .map(|s| s.as_str())
        .collect();

    let user_topic_vec = compute_need_vector(user_posts.to_vec());
    let mentor_topic_vec = compute_experience_vector(mentor_posts.to_vec());



    // for testing init topic vectors
    //let user_topic_vec = vec![Topics::Housing, Topics::VisaAdmin, Topics::CampusLogistics];
    //let mentor_topic_vec = vec![Topics::VisaAdmin, Topics::CampusLogistics, Topics::MoneyWork, Topics::Coursework, Topics::StudyHelp, Topics::SocialBelonging, Topics::MentalHealth, Topics::LanguageSupport];

    // shadow as type of TopicVector
    //let user_topic_vec: TopicVector = build_topic_vector(user_topic_vec);
    //let mentor_topic_vec: TopicVector = build_topic_vector(mentor_topic_vec); 
    // user origin
    //let user_origin = Country::Other;
    //let mentor_origin = Country::India;
    
    // user hobbies -> will need some cleaning before maybe enforce on AT proto data
    // case much match exactly as of now
    //let user_hobbies = vec!["cars", "cooking", "sports", "music", "coffee"];
    //let mentor_hobbies = vec!["music", "cooking", "sports", "cars", "coffee"];
    

    //let user_subjects = vec!["linear algebra", "calculus", "physics", "english"];
    //let user_subjects = vec!["computer science", "physics", "calculus", "math", "science"];
    //let mentor_subjects = vec!["computer science", "physics", "calculus", "math", "science"];
    // topic sim = -9 to 9.. -9 means 9 needs.. mentor has 0 experience
    // this is most important value...
    // add weights to all... not all maxed at 1.. 0 = min
    let topic_sim: f64 = 0.40 * topic_similarity(user_topic_vec, mentor_topic_vec);
    let origin_sim: f64 = 0.15 * origin_match(user_origin, mentor_origin);
    let hobby_sim: f64 = 0.25 * hobby_overlap(user_hobbies, mentor_hobbies);
    let subject_sim: f64 = 0.20 * subject_similarity(user_subjects, mentor_subjects);
    

    let mms = topic_sim + origin_sim + hobby_sim + subject_sim;
    println!("{} score = {}t + {}o + {}h + {}s", mms, topic_sim, origin_sim, hobby_sim, subject_sim);
    mms
}


/// TOPIC - keyword Dispatch table
pub static TOPIC_KEYWORD_TABLE: &[(Topics, &[&str])] = &[
    (Topics::Housing, HOUSING_KEYWORDS),
    (Topics::VisaAdmin, VISA_ADMIN_KEYWORDS),
    (Topics::Coursework, COURSEWORK_KEYWORDS),
    (Topics::StudyHelp, STUDY_HELP_KEYWORDS),
    (Topics::SocialBelonging, SOCIAL_BELONGING_KEYWORDS),
    (Topics::MentalHealth, MENTAL_HEALTH_KEYWORDS),
    (Topics::CampusLogistics, CAMPUS_LOGISTICS_KEYWORDS),
    (Topics::MoneyWork, MONEY_WORK_KEYWORDS),
    (Topics::LanguageSupport, LANGUAGE_SUPPORT_KEYWORDS),
];


// ===============================
// Topic Keyword Dictionaries
// - Below was Vibecoded..
//   not typing all that out
// ===============================

pub static HOUSING_KEYWORDS: &[&str] = &[
    "housing", "room", "dorm", "apartment", "rent", "lease", "sublet", "move-in",
    "move out", "roommate", "landlord", "utilities", "water bill", "maintenance",
    "furniture", "off-campus", "on-campus", "residence hall", "housing office",
];

pub static VISA_ADMIN_KEYWORDS: &[&str] = &[
    "visa", "f1", "i-20", "sevis", "immigration", "passport", "status",
    "work authorization", "opt", "cpt", "international office",
    "document check", "uscis", "appointment", "check-in",
];

pub static COURSEWORK_KEYWORDS: &[&str] = &[
    "coursework", "class", "assignment", "homework", "project", "exam",
    "midterm", "final", "lecture", "syllabus", "quiz", "grade", "lab",
    "professor", "canvas", "blackboard", "group project",
];

pub static STUDY_HELP_KEYWORDS: &[&str] = &[
    "study", "tutor", "help", "explain", "confused", "practice",
    "office hours", "notes", "review", "problem set", "study group",
    "exam prep", "solutions", "walkthrough",
];

pub static SOCIAL_BELONGING_KEYWORDS: &[&str] = &[
    "friend", "friends", "meet people", "lonely", "community", "events",
    "club", "organization", "hang out", "party", "connect", "social",
    "orientation", "icebreaker", "belonging",
];

pub static MENTAL_HEALTH_KEYWORDS: &[&str] = &[
    "stress", "anxiety", "overwhelmed", "depressed", "burnout",
    "panic", "mental health", "counseling", "therapy", "exhausted",
    "sleep", "tired", "nervous", "homesick", "pressure",
];

pub static CAMPUS_LOGISTICS_KEYWORDS: &[&str] = &[
    "parking", "shuttle", "bus", "campus map", "library", "wifi",
    "id card", "meal plan", "hours", "building", "directions",
    "printing", "facilities", "maintenance", "dining hall",
];

pub static MONEY_WORK_KEYWORDS: &[&str] = &[
    "job", "work", "on-campus job", "paycheck", "bank", "financial",
    "budget", "money", "scholarship", "assistantship", "tuition",
    "fees", "hire", "internship", "resume", "career fair",
];

pub static LANGUAGE_SUPPORT_KEYWORDS: &[&str] = &[
    "english", "language", "translate", "translation", "speaking",
    "communication", "accent", "understand", "pronunciation",
    "writing help", "conversation practice",
];

