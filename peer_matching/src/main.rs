use std::vec;
use std::iter::zip;

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



fn main() {
    let match_score = mentor_match_score();
    println!("{}", match_score);

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
fn topic_similarity(user_topic_vector: TopicVector, mentor_topic_vector: TopicVector) -> u8 {
    // count where user and Not mentor.. so user has a need mentor has no exp with
    let penalty = zip(user_topic_vector.0.iter(), mentor_topic_vector.0.iter())
        .filter(|&(&user_topic_vector, &mentor_topic_vector)| user_topic_vector && !mentor_topic_vector)
        .count() as u8;

    // count where user and mentor share a topic
    let matches = zip(user_topic_vector.0.iter(), mentor_topic_vector.0.iter())
        .filter(|&(&user_topic_vector, &mentor_topic_vector)| user_topic_vector && mentor_topic_vector)
        .count() as u8;

    // value = match - penalty -- improve by normalizing values to only Mentee Needs
    let result = matches - penalty;
    return result;
}

fn origin_match(user_origin: Country, mentor_origin: Country) {
    // TODO HERE
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
fn mentor_match_score() -> f64 {
   
    // for testing init topic vectors
    let user_topic_vec = vec![Topics::Housing, Topics::VisaAdmin, Topics::CampusLogistics];
    let mentor_topic_vec = vec![Topics::VisaAdmin, Topics::CampusLogistics, Topics::MoneyWork];
    
    // shadow as type of TopicVector
    let user_topic_vec: TopicVector = build_topic_vector(user_topic_vec);
    let mentor_topic_vec: TopicVector = build_topic_vector(mentor_topic_vec); 

    // topic sim = -9 to 9.. -9 means 9 needs.. mentor has 0 experience
    // this is most important value...
    let topic_sim = f64::from(topic_similarity(user_topic_vec, mentor_topic_vec));
    


    let mms = topic_sim;
    mms
}
