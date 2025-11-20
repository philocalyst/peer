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

/// user seeking help/their info
struct Mentee {}

/// user flagged as offering help
struct Mentor {}

fn main() {
    let match_score = mentor_match_score();
    println!("{}", match_score);

}

fn build_topic_vector() ->  {
}


fn topic_similarity(user_topic_vec: Vec<Topics>, mentor_topic_vec: Vec<Topics>) -> f64 {
    // cosine simmilarity for user, mentor topic vectors
    return 0.35f64;
}


fn mentor_match_score() -> f64 {
    /// score for any user, mentor pairing -> mentors must have mentor tag
    ///     OR meet a minimum number of qualifications.
    /// Calculates score based on 
    /// topic match - cosine similarity
    /// origin match -> yes/no
    /// interest_overlap - Jaccard
    /// study Subjusts - Jaccard 
    /// language score -> 1 = match 1 languge, 0 else
    /// helpful_score -> ratio of endorsements/posts
    
    // for testing init topic vectors
    let user_topic_vec = vec![Topics::Housing, Topics::VisaAdmin, Topics::CampusLogistics];
    let mentor_topic_vec = vec![Topics::VisaAdmin, Topics::CampusLogistics, Topics::MoneyWork];

    let topic_sim = topic_similarity(user_topic_vec, mentor_topic_vec); 
    
    let mms = topic_sim;
    mms
}
