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
#[derive(Copy, Clone)]
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
    UnitedStates,
    Spain,
    Other,   // catch-all fallback
}

#[derive(PartialEq, Eq, Clone)]
pub enum UserRole {
    Peer,
    Mentor,
    Mentee,
}

/// Define Structs -> UserInfo = static
/// Post -> Ref User, sometimes changes
/// UserState -> Derived topics/Needs/Experience Vectors
#[derive(Clone)]
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
pub fn mentor_match_score(user_info: &UserInfo, mentor_info: &UserInfo, user_topic_vec: TopicVector, mentor_topic_vec: TopicVector,) -> f64 {
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

    //let user_topic_vec = compute_need_vector(user_posts.to_vec());
    //let mentor_topic_vec = compute_experience_vector(mentor_posts.to_vec());



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



/// TEST USERS TO SEED 
pub fn seed_users() -> Vec<UserInfo> {
    vec![

        // --- MENTEES ---

        UserInfo {
            did: "did:peerlink:jerry".to_string(),
            display_name: "Jerry".to_string(),
            role: UserRole::Mentee,
            country: Country::UnitedStates, // <-- You need to add this to enum or use Other
            study_subjects: vec![
                "computer science".to_string(),
                "calculus".to_string(),
                "english composition".to_string(),
            ],
            hobbies: vec![
                "music".to_string(),
                "gaming".to_string(),
                "coffee".to_string(),
            ],
            languages: vec!["english".to_string()],
            arrival_date: "2025-08-15T00:00:00Z".to_string(),
        },

        UserInfo {
            did: "did:peerlink:raheem".to_string(),
            display_name: "Raheem".to_string(),
            role: UserRole::Mentee,
            country: Country::Nigeria,
            study_subjects: vec![
                "electrical engineering".to_string(),
                "calculus".to_string(),
                "physics".to_string(),
            ],
            hobbies: vec![
                "soccer".to_string(),
                "cooking".to_string(),
                "photography".to_string(),
            ],
            languages: vec!["english".to_string(), "yoruba".to_string()],
            arrival_date: "2025-08-10T00:00:00Z".to_string(),
        },

        UserInfo {
            did: "did:peerlink:lina".to_string(),
            display_name: "Lina".to_string(),
            role: UserRole::Mentee,
            country: Country::Vietnam,
            study_subjects: vec![
                "biology".to_string(),
                "chemistry".to_string(),
                "statistics".to_string(),
            ],
            hobbies: vec![
                "reading".to_string(),
                "walking".to_string(),
                "lofi music".to_string(),
            ],
            languages: vec!["vietnamese".to_string(), "english".to_string()],
            arrival_date: "2025-08-12T00:00:00Z".to_string(),
        },

        UserInfo {
            did: "did:peerlink:carlos".to_string(),
            display_name: "Carlos".to_string(),
            role: UserRole::Mentee,
            country: Country::Mexico,
            study_subjects: vec![
                "mechanical engineering".to_string(),
                "calculus".to_string(),
                "materials science".to_string(),
            ],
            hobbies: vec![
                "soccer".to_string(),
                "gym".to_string(),
                "cooking".to_string(),
            ],
            languages: vec!["spanish".to_string(), "english".to_string()],
            arrival_date: "2025-08-18T00:00:00Z".to_string(),
        },

        // --- MENTORS ---

        UserInfo {
            did: "did:peerlink:sofia".to_string(),
            display_name: "Sofia".to_string(),
            role: UserRole::Mentor,
            country: Country::Spain, // <-- You need to add Spain to your enum or map to Other
            study_subjects: vec![
                "computer science".to_string(),
                "human-computer interaction".to_string(),
            ],
            hobbies: vec![
                "music".to_string(),
                "student clubs".to_string(),
                "baking".to_string(),
            ],
            languages: vec!["spanish".to_string(), "english".to_string()],
            arrival_date: "2022-08-15T00:00:00Z".to_string(),
        },

        UserInfo {
            did: "did:peerlink:aisha".to_string(),
            display_name: "Aisha".to_string(),
            role: UserRole::Mentor,
            country: Country::Pakistan,
            study_subjects: vec![
                "information systems".to_string(),
                "economics".to_string(),
            ],
            hobbies: vec![
                "budgeting tips".to_string(),
                "coffee".to_string(),
                "podcasts".to_string(),
            ],
            languages: vec!["urdu".to_string(), "english".to_string()],
            arrival_date: "2021-08-20T00:00:00Z".to_string(),
        },

        UserInfo {
            did: "did:peerlink:mei".to_string(),
            display_name: "Mei".to_string(),
            role: UserRole::Mentor,
            country: Country::China,
            study_subjects: vec![
                "linguistics".to_string(),
                "education".to_string(),
            ],
            hobbies: vec![
                "language exchange".to_string(),
                "tea".to_string(),
                "campus events".to_string(),
            ],
            languages: vec!["mandarin".to_string(), "english".to_string()],
            arrival_date: "2022-01-10T00:00:00Z".to_string(),
        },

        UserInfo {
            did: "did:peerlink:daniel".to_string(),
            display_name: "Daniel".to_string(),
            role: UserRole::Mentor,
            country: Country::Canada,
            study_subjects: vec![
                "psychology".to_string(),
                "neuroscience".to_string(),
            ],
            hobbies: vec![
                "running".to_string(),
                "mindfulness".to_string(),
                "board games".to_string(),
            ],
            languages: vec!["english".to_string(), "french".to_string()],
            arrival_date: "2021-08-25T00:00:00Z".to_string(),
        },
    ]
}


/// SEED posts -- shared with frontend
/// posts gen by ai to represent personas
pub fn seed_posts() -> Vec<Post> {
    vec![

        // ---------- JERRY ----------
        Post {
            author_id: "did:peerlink:jerry".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "I'm a first-year CS student and I'm already confused by the first programming assignment. \
                   The lecture made sense but the project instructions on Canvas feel vague. \
                   Does anyone have notes or a study group for this class or tips for office hours?"
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:jerry".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "I’d like to meet people but I’m not sure which clubs or events are friendly for new students. \
                   I feel a bit lonely in the dorm and don’t know where people usually hang out after class."
                .to_string(),
            topics: vec![],
        },

        // ---------- RAHEEM ----------
        Post {
            author_id: "did:peerlink:raheem".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "Hi, I just arrived on my F1 visa and I'm confused about the SEVIS check-in. \
                   Do I need to bring my passport, I-20 and admission letter to the international office, \
                   and how do I book the appointment?"
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:raheem".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "I'm looking for housing near campus. Off-campus apartments and landlords are asking about the lease \
                   and utilities and I don't understand the contract. Any advice from other international students?"
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:raheem".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "Also, how do you figure out the campus shuttle and bus routes? \
                   I keep getting lost finding the right building and the library hours."
                .to_string(),
            topics: vec![],
        },

        // ---------- LINA ----------
        Post {
            author_id: "did:peerlink:lina".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "I’m feeling really overwhelmed balancing biology lab reports, chemistry homework, and statistics quizzes. \
                   I’m constantly stressed and barely sleeping, and I’m worried about burning out. \
                   How do you manage study schedule and still get enough rest?"
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:lina".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "Does the campus counseling or mental health service actually help with anxiety and panic before exams? \
                   I’m nervous to book therapy but I think I need support."
                .to_string(),
            topics: vec![],
        },

        // ---------- CARLOS ----------
        Post {
            author_id: "did:peerlink:carlos".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "I'm trying to find an on-campus job or work-study position to help with tuition and fees. \
                   Where do you usually find job postings and how does the paycheck and budget work for international students?"
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:carlos".to_string(),
            kind: PostKind::HelpAsk,
            flagged: false,
            text: "Also, I'm confused about the meal plan and dining hall hours. \
                   I keep arriving when the dining hall is closed and end up spending extra money off-campus."
                .to_string(),
            topics: vec![],
        },

        // ---------- SOFIA ----------
        Post {
            author_id: "did:peerlink:sofia".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "For intro CS, I recommend forming a small study group and using office hours twice a week. \
                   Bring your assignment questions, and ask the professor to walk you through sample problems before the exam. \
                   Canvas usually has old quizzes and review notes you can practice with."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:sofia".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "If you feel lonely in the dorm, try the computer science club and the international student events during orientation. \
                   Those are low-pressure ways to meet friends who are also looking for community."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:sofia".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "On housing: before signing a lease, check that rent, utilities, and move-in dates are clearly written. \
                   The housing office can review your apartment contract if you're unsure about the landlord."
                .to_string(),
            topics: vec![],
        },

        // ---------- AISHA ----------
        Post {
            author_id: "did:peerlink:aisha".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "For F1 visa students, you must complete SEVIS check-in within the first 30 days. \
                   Bring your passport, I-20, and admission letter to the international office. \
                   They'll update your immigration status and confirm your work authorization timeline."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:aisha".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "On-campus jobs are usually listed on the student employment portal. \
                   Look for assistantship or library positions—they respect your course schedule. \
                   Plan a simple budget for tuition, housing, food, and transportation so you don't stress over every paycheck."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:aisha".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "For campus logistics, the shuttle schedule and bus routes are on the university app. \
                   Save your favorite buildings and the library hours so you don’t waste time walking back and forth."
                .to_string(),
            topics: vec![],
        },

        // ---------- MEI ----------
        Post {
            author_id: "did:peerlink:mei".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "If you're anxious about speaking English, try the language exchange club. \
                   We practice conversation, pronunciation, and communication in a relaxed setting, \
                   and you can also help others learn your language."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:mei".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "It’s normal to feel nervous about your accent or writing. \
                   The writing center and conversation practice groups can help with translation, grammar, and confidence. \
                   You’ll meet friends who share the same language challenges."
                .to_string(),
            topics: vec![],
        },

        // ---------- DANIEL ----------
        Post {
            author_id: "did:peerlink:daniel".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "Feeling overwhelmed is common in the first semester. \
                   Try blocking your week into study sessions and rest windows. \
                   Protect your sleep by turning off notifications late at night—chronic sleep loss makes stress and anxiety worse."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:daniel".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "The counseling center offers short, focused therapy for anxiety and panic before exams. \
                   You can also attend workshops on burnout and time management. \
                   They're confidential and designed specifically for students under pressure."
                .to_string(),
            topics: vec![],
        },
        Post {
            author_id: "did:peerlink:daniel".to_string(),
            kind: PostKind::HelpReply,
            flagged: false,
            text: "If you keep missing dining hall hours, check the campus app and set reminders. \
                   Having a predictable routine for meals and study locations reduces decision fatigue and stress."
                .to_string(),
            topics: vec![],
        },
    ]
}

