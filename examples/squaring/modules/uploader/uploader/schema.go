package uploader

type Staker struct {
	TotalAmount float64
	Strategies  []Strategy
}

type Strategy struct {
	Strategy string
	Amount   float64
}

type Submission struct {
	Strategy string
	Token    string
	Amount   float64
}

type Earner struct {
	Earner           string
	TotalStakeAmount float64
	Tokens           []*TokenAmount
}

type TokenAmount struct {
	Strategy     string
	Token        string
	RewardAmount string
	StakeAmount  float64
}
