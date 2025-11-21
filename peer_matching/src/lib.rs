use std::vec;
use std::iter::zip;
use std::collections::HashSet;

/// topics
///
/// for classifying posts
/// for identifying matches
enum Topics {
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
struct TopicVector([bool; 9]);
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


/// Define Structs -> UserInfo = static
/// Post -> Ref User, sometimes changes
/// UserState -> Derived topics/Needs/Experience Vectors
struct UserInfo {
    

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
    // count where user and Not mentor.. so user has a need mentor has no exp with
    let penalty = zip(user_topic_vector.0.iter(), mentor_topic_vector.0.iter())
        .filter(|&(&user_topic_vector, &mentor_topic_vector)| user_topic_vector && !mentor_topic_vector)
        .count() as u8;

    // count where user and mentor share a topic
    let matches = zip(user_topic_vector.0.iter(), mentor_topic_vector.0.iter())
        .filter(|&(&user_topic_vector, &mentor_topic_vector)| user_topic_vector && mentor_topic_vector)
        .count() as u8;
    
    let user_topic_count = user_topic_vector.0
        .iter()
        .filter(|&user_topic_vector| *user_topic_vector)
        .count() as f64;
    // value = match - penalty -- improve by normalizing values to only Mentee Needs
    let result = (matches - penalty) as f64 / user_topic_count;

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
fn jaccard_similarity(a: &Vec<&str>, b: &Vec<&str>) -> f64 {
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
pub fn mentor_match_score() -> f64 {

    // for testing init topic vectors
    let user_topic_vec = vec![Topics::Housing, Topics::VisaAdmin, Topics::CampusLogistics];
    let mentor_topic_vec = vec![Topics::VisaAdmin, Topics::CampusLogistics, Topics::MoneyWork, Topics::Coursework, Topics::StudyHelp, Topics::SocialBelonging, Topics::MentalHealth, Topics::LanguageSupport];

    // shadow as type of TopicVector
    let user_topic_vec: TopicVector = build_topic_vector(user_topic_vec);
    let mentor_topic_vec: TopicVector = build_topic_vector(mentor_topic_vec); 
    // user origin
    let user_origin = Country::Other;
    let mentor_origin = Country::India;
    
    // user hobbies -> will need some cleaning before maybe enforce on AT proto data
    // case much match exactly as of now
    let user_hobbies = vec!["cars", "cooking", "sports", "music", "coffee"];
    let mentor_hobbies = vec!["music", "cooking", "sports", "cars", "coffee"];
    

    //let user_subjects = vec!["linear algebra", "calculus", "physics", "english"];
    let user_subjects = vec!["computer science", "physics", "calculus", "math", "science"];
    let mentor_subjects = vec!["computer science", "physics", "calculus", "math", "science"];
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

