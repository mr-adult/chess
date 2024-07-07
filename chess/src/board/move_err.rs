#[derive(Debug)]
pub enum MoveErr {
    IllegalMove,
    NoPieceAtFromLocation,
    IllegalPromotionPieceChoice,
    PromotionTargetNotPawn,
    MislabeledPromotion,
}
