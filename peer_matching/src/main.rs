use std::vec;


/// topics
///
/// for classifying posts
/// for identifying matches
///
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


fn topic_similarity(user_topic_vector: TopicVector, mentor_topic_vector: TopicVector) -> f64 {
    // cosine simmilarity for user, mentor topic vectors
    return 0.35f64;
}


/// score for any user, mentor pairing -> mentors must have mentor tag
///     OR meet a minimum number of qualifications.
/// Calculates score based on 
/// topic match - cosine similarity
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

    let topic_sim = topic_similarity(user_topic_vec, mentor_topic_vec); 
    
    let mms = topic_sim;
    mms
}
