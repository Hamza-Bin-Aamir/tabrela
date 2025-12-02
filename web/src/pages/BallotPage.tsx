import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { TabulationService } from '../services/tabulation';
import type {
  MatchResponse,
  BallotResponse,
  SpeakerScoreInput,
  TeamRankingInput,
  MatchTeam,
} from '../services/types';

interface SpeakerScoreState {
  allocation_id: string;
  username: string;
  team_position: string;
  role: string;
  score: number;
  feedback: string;
}

interface TeamRankState {
  team_id: string;
  position: string;
  rank: number;
  is_winner: boolean;
}

export default function BallotPage() {
  const { matchId } = useParams<{ matchId: string }>();
  const navigate = useNavigate();
  
  const [match, setMatch] = useState<MatchResponse | null>(null);
  const [existingBallot, setExistingBallot] = useState<BallotResponse | null>(null);
  const [speakerScores, setSpeakerScores] = useState<SpeakerScoreState[]>([]);
  const [teamRanks, setTeamRanks] = useState<TeamRankState[]>([]);
  const [notes, setNotes] = useState('');
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isVotingAdjudicator, setIsVotingAdjudicator] = useState(false);

  const loadMatchData = useCallback(async () => {
    if (!matchId) return;
    
    try {
      setLoading(true);
      
      // Get match details
      const matchData = await TabulationService.getMatch(matchId);
      setMatch(matchData);
      
      // Try to get existing ballot
      try {
        const ballotData = await TabulationService.getMyBallot(matchId);
        setExistingBallot(ballotData);
        setIsVotingAdjudicator(ballotData.is_voting);
        setNotes(ballotData.notes || '');
        
        // Initialize from existing ballot
        if (ballotData.speaker_scores.length > 0) {
          // Map existing scores
          const existingScoresMap = new Map(
            ballotData.speaker_scores.map(s => [s.allocation_id, s])
          );
          
          const scores: SpeakerScoreState[] = [];
          matchData.teams.forEach(team => {
            const position = team.four_team_position || team.two_team_position || '';
            team.speakers.forEach(speaker => {
              const existing = existingScoresMap.get(speaker.allocation_id);
              scores.push({
                allocation_id: speaker.allocation_id,
                username: speaker.username,
                team_position: position.replace(/_/g, ' '),
                role: (speaker.four_team_speaker_role || speaker.two_team_speaker_role || '').replace(/_/g, ' '),
                score: existing?.score || 75,
                feedback: existing?.feedback || '',
              });
            });
          });
          setSpeakerScores(scores);
        } else {
          initializeSpeakerScores(matchData.teams);
        }
        
        if (ballotData.team_rankings.length > 0) {
          const existingRanksMap = new Map(
            ballotData.team_rankings.map(r => [r.team_id, r])
          );
          
          const ranks: TeamRankState[] = matchData.teams.map(team => {
            const existing = existingRanksMap.get(team.id);
            return {
              team_id: team.id,
              position: (team.four_team_position || team.two_team_position || team.team_name || '').replace(/_/g, ' '),
              rank: existing?.rank || 0,
              is_winner: existing?.is_winner || false,
            };
          });
          setTeamRanks(ranks);
        } else {
          initializeTeamRanks(matchData.teams);
        }
      } catch {
        // No existing ballot - initialize fresh
        initializeSpeakerScores(matchData.teams);
        initializeTeamRanks(matchData.teams);
        // Check if current user is a voting adjudicator from match data
        // This would require auth context - for now default to true
        setIsVotingAdjudicator(true);
      }

      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load match data');
    } finally {
      setLoading(false);
    }
  }, [matchId]);

  const initializeSpeakerScores = (teams: MatchTeam[]) => {
    const scores: SpeakerScoreState[] = [];
    teams.forEach(team => {
      const position = team.four_team_position || team.two_team_position || '';
      team.speakers.forEach(speaker => {
        scores.push({
          allocation_id: speaker.allocation_id,
          username: speaker.username,
          team_position: position.replace(/_/g, ' '),
          role: (speaker.four_team_speaker_role || speaker.two_team_speaker_role || '').replace(/_/g, ' '),
          score: 75, // Default middle score
          feedback: '',
        });
      });
    });
    setSpeakerScores(scores);
  };

  const initializeTeamRanks = (teams: MatchTeam[]) => {
    const ranks: TeamRankState[] = teams.map((team, index) => ({
      team_id: team.id,
      position: (team.four_team_position || team.two_team_position || team.team_name || '').replace(/_/g, ' '),
      rank: index + 1,
      is_winner: index === 0,
    }));
    setTeamRanks(ranks);
  };

  useEffect(() => {
    loadMatchData();
  }, [loadMatchData]);

  const handleScoreChange = (allocationId: string, value: number) => {
    setSpeakerScores(prev => prev.map(s => 
      s.allocation_id === allocationId ? { ...s, score: value } : s
    ));
  };

  const handleFeedbackChange = (allocationId: string, value: string) => {
    setSpeakerScores(prev => prev.map(s => 
      s.allocation_id === allocationId ? { ...s, feedback: value } : s
    ));
  };

  const handleRankChange = (teamId: string, newRank: number) => {
    setTeamRanks(prev => {
      // Find the team that currently has this rank
      const teamWithNewRank = prev.find(t => t.rank === newRank && t.team_id !== teamId);
      const teamBeingChanged = prev.find(t => t.team_id === teamId);
      
      if (!teamBeingChanged) return prev;
      
      const oldRank = teamBeingChanged.rank;
      
      return prev.map(t => {
        if (t.team_id === teamId) {
          return { ...t, rank: newRank, is_winner: newRank === 1 };
        }
        if (teamWithNewRank && t.team_id === teamWithNewRank.team_id) {
          return { ...t, rank: oldRank, is_winner: oldRank === 1 };
        }
        return t;
      });
    });
  };

  const validateBallot = (): string | null => {
    // Check all scores are in valid range
    for (const score of speakerScores) {
      if (score.score < 50 || score.score > 100) {
        return `Score for ${score.username} must be between 50 and 100`;
      }
    }

    // Check rankings are valid (1 to n without duplicates)
    const ranks = teamRanks.map(t => t.rank).sort((a, b) => a - b);
    for (let i = 0; i < ranks.length; i++) {
      if (ranks[i] !== i + 1) {
        return 'Each team must have a unique rank from 1 to ' + ranks.length;
      }
    }

    return null;
  };

  const handleSubmit = async () => {
    if (!matchId) return;
    
    const validationError = validateBallot();
    if (validationError) {
      setError(validationError);
      return;
    }

    try {
      setSubmitting(true);
      setError(null);

      const speakerScoreInputs: SpeakerScoreInput[] = speakerScores.map(s => ({
        allocation_id: s.allocation_id,
        score: s.score,
        feedback: s.feedback || undefined,
      }));

      const teamRankingInputs: TeamRankingInput[] = teamRanks.map(t => ({
        team_id: t.team_id,
        rank: t.rank,
        is_winner: t.is_winner,
      }));

      if (isVotingAdjudicator) {
        await TabulationService.submitBallot({
          match_id: matchId,
          notes: notes || undefined,
          speaker_scores: speakerScoreInputs,
          team_rankings: teamRankingInputs,
        });
      } else {
        await TabulationService.submitFeedback({
          match_id: matchId,
          notes: notes,
        });
      }

      navigate(`/matches/${matchId}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit ballot');
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error && !match) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
          {error}
        </div>
      </div>
    );
  }

  if (!match) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
          Match not found
        </div>
      </div>
    );
  }

  const isFourTeam = match.teams.some(t => t.four_team_position);
  
  // Separate teams into government and opposition sides
  const govTeams = match.teams.filter(t => 
    t.two_team_position === 'government' || 
    t.four_team_position?.includes('government')
  );
  const oppTeams = match.teams.filter(t => 
    t.two_team_position === 'opposition' || 
    t.four_team_position?.includes('opposition')
  );

  const getTeamLabel = (team: typeof match.teams[0]) => {
    if (team.two_team_position === 'government') return '🏛️ Government';
    if (team.two_team_position === 'opposition') return '⚔️ Opposition';
    if (team.four_team_position === 'opening_government') return '🏛️ Opening Government';
    if (team.four_team_position === 'closing_government') return '🏛️ Closing Government';
    if (team.four_team_position === 'opening_opposition') return '⚔️ Opening Opposition';
    if (team.four_team_position === 'closing_opposition') return '⚔️ Closing Opposition';
    return team.team_name || 'Team';
  };

  const renderTeamCard = (team: typeof match.teams[0], isGov: boolean) => {
    const teamSpeakers = speakerScores.filter(s => 
      team.speakers.some(sp => sp.allocation_id === s.allocation_id)
    );

    return (
      <div 
        key={team.id} 
        className={`rounded-lg border-2 p-4 ${
          isGov ? 'border-blue-300 bg-blue-50' : 'border-purple-300 bg-purple-50'
        }`}
      >
        <h3 className={`font-semibold text-lg mb-4 ${isGov ? 'text-blue-800' : 'text-purple-800'}`}>
          {getTeamLabel(team)}
        </h3>
        
        <div className="space-y-4">
          {teamSpeakers.length > 0 ? teamSpeakers.map(speaker => (
            <div key={speaker.allocation_id} className="bg-white rounded-lg p-4 shadow-sm">
              <div className="flex items-center justify-between mb-3">
                <div>
                  <span className="font-medium text-gray-900">{speaker.username}</span>
                  <span className="text-sm text-gray-500 ml-2 capitalize">
                    ({speaker.role.replace(/_/g, ' ')})
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-sm text-gray-500">Score:</span>
                  <input
                    type="number"
                    min={50}
                    max={100}
                    step={0.5}
                    value={speaker.score}
                    onChange={(e) => handleScoreChange(speaker.allocation_id, parseFloat(e.target.value) || 50)}
                    className={`w-20 px-3 py-2 border-2 rounded-lg text-center font-bold text-lg ${
                      speaker.score >= 80 ? 'border-green-400 bg-green-50 text-green-700' :
                      speaker.score >= 75 ? 'border-blue-400 bg-blue-50 text-blue-700' :
                      speaker.score >= 70 ? 'border-yellow-400 bg-yellow-50 text-yellow-700' :
                      'border-red-400 bg-red-50 text-red-700'
                    }`}
                  />
                </div>
              </div>
              
              <textarea
                placeholder="Feedback for this speaker (optional)"
                value={speaker.feedback}
                onChange={(e) => handleFeedbackChange(speaker.allocation_id, e.target.value)}
                rows={2}
                className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm resize-none focus:ring-2 focus:ring-blue-300 focus:border-transparent"
              />
            </div>
          )) : (
            <p className="text-sm text-gray-500 italic">No speakers allocated</p>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="max-w-6xl mx-auto px-4 py-4 sm:px-6 lg:px-8">
          <h1 className="text-2xl font-bold text-gray-900">
            {isVotingAdjudicator ? 'Submit Ballot' : 'Submit Feedback'}
          </h1>
          <p className="text-sm text-gray-500 mt-1">
            {match.series_name} • Room: {match.room_name || 'TBA'}
          </p>
          {existingBallot?.is_submitted && (
            <div className="mt-2 text-sm text-yellow-600 bg-yellow-50 px-3 py-1 rounded inline-block">
              You have already submitted a ballot. Submitting again will update it.
            </div>
          )}
        </div>
      </header>

      <main className="max-w-6xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
            {error}
          </div>
        )}

        {/* Motion */}
        {match.motion && (
          <div className="bg-white rounded-lg shadow p-6 mb-6">
            <h2 className="text-lg font-semibold mb-2">Motion</h2>
            <p className="text-gray-700 text-lg">{match.motion}</p>
            {match.info_slide && (
              <div className="mt-4 p-4 bg-gray-50 rounded-lg">
                <h3 className="text-sm font-medium text-gray-500 mb-1">Info Slide</h3>
                <p className="text-gray-700">{match.info_slide}</p>
              </div>
            )}
          </div>
        )}

        {/* Speaker Scores - Only for voting adjudicators */}
        {isVotingAdjudicator && (
          <div className="bg-white rounded-lg shadow p-6 mb-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Speaker Scores</h2>
              <div className="flex items-center gap-4 text-sm text-gray-500">
                <span className="flex items-center gap-1">
                  <span className="w-3 h-3 rounded bg-green-400"></span> 80+ Excellent
                </span>
                <span className="flex items-center gap-1">
                  <span className="w-3 h-3 rounded bg-blue-400"></span> 75-79 Good
                </span>
                <span className="flex items-center gap-1">
                  <span className="w-3 h-3 rounded bg-yellow-400"></span> 70-74 Average
                </span>
                <span className="flex items-center gap-1">
                  <span className="w-3 h-3 rounded bg-red-400"></span> &lt;70 Below
                </span>
              </div>
            </div>
            
            {/* Two-column layout: Gov on left, Opp on right */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Government Side */}
              <div className="space-y-4">
                <h3 className="text-sm font-medium text-blue-600 uppercase tracking-wider">Government Bench</h3>
                {govTeams.map(team => renderTeamCard(team, true))}
              </div>

              {/* Opposition Side */}
              <div className="space-y-4">
                <h3 className="text-sm font-medium text-purple-600 uppercase tracking-wider">Opposition Bench</h3>
                {oppTeams.map(team => renderTeamCard(team, false))}
              </div>
            </div>
          </div>
        )}

        {/* Team Rankings - Only for voting adjudicators in 4-team format */}
        {isVotingAdjudicator && isFourTeam && (
          <div className="bg-white rounded-lg shadow p-6 mb-6">
            <h2 className="text-lg font-semibold mb-4">Team Rankings</h2>
            <p className="text-sm text-gray-500 mb-4">
              Rank each team from 1st to 4th place.
            </p>
            
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {teamRanks.sort((a, b) => a.rank - b.rank).map(team => (
                <div 
                  key={team.team_id} 
                  className={`rounded-lg p-4 border-2 text-center ${
                    team.rank === 1 ? 'border-yellow-400 bg-yellow-50' :
                    team.rank === 2 ? 'border-gray-400 bg-gray-50' :
                    team.rank === 3 ? 'border-orange-400 bg-orange-50' :
                    'border-gray-200 bg-white'
                  }`}
                >
                  <div className="text-sm font-medium mb-2 capitalize">
                    {team.position}
                  </div>
                  <select
                    value={team.rank}
                    onChange={(e) => handleRankChange(team.team_id, parseInt(e.target.value))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md text-center font-medium"
                  >
                    {[1, 2, 3, 4].map(r => (
                      <option key={r} value={r}>{r}{r === 1 ? 'st' : r === 2 ? 'nd' : r === 3 ? 'rd' : 'th'}</option>
                    ))}
                  </select>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 2-Team Winner Selection */}
        {isVotingAdjudicator && !isFourTeam && teamRanks.length === 2 && (
          <div className="bg-white rounded-lg shadow p-6 mb-6">
            <h2 className="text-lg font-semibold mb-4">Select Winner</h2>
            <div className="grid grid-cols-2 gap-4">
              {teamRanks.map(team => (
                <button
                  key={team.team_id}
                  onClick={() => {
                    // Set this team as rank 1 (winner), other as rank 2
                    setTeamRanks(prev => prev.map(t => ({
                      ...t,
                      rank: t.team_id === team.team_id ? 1 : 2,
                      is_winner: t.team_id === team.team_id,
                    })));
                  }}
                  className={`p-6 rounded-lg border-2 text-center transition-all ${
                    team.rank === 1
                      ? 'border-green-500 bg-green-50 ring-2 ring-green-300'
                      : 'border-gray-200 bg-white hover:border-gray-300'
                  }`}
                >
                  <div className={`text-lg font-semibold ${team.rank === 1 ? 'text-green-700' : 'text-gray-700'}`}>
                    {team.position.includes('government') ? '🏛️ Government' : '⚔️ Opposition'}
                  </div>
                  {team.rank === 1 && (
                    <div className="mt-2 text-green-600 font-medium">✓ Winner</div>
                  )}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Notes */}
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <h2 className="text-lg font-semibold mb-4">
            {isVotingAdjudicator ? 'Private Notes' : 'Feedback'}
          </h2>
          <p className="text-sm text-gray-500 mb-4">
            {isVotingAdjudicator 
              ? 'These notes are private and visible only to admins.'
              : 'Provide your overall feedback on the debate.'}
          </p>
          
          <textarea
            placeholder={isVotingAdjudicator ? 'Add private notes...' : 'Your feedback...'}
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            rows={4}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg resize-none focus:ring-2 focus:ring-blue-300 focus:border-transparent"
          />
        </div>

        {/* Submit Button */}
        <div className="flex justify-end gap-4">
          <button
            onClick={() => navigate(-1)}
            className="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={submitting}
            className={`px-6 py-2 rounded-lg text-white font-medium transition-colors ${
              submitting 
                ? 'bg-blue-400 cursor-not-allowed' 
                : 'bg-blue-600 hover:bg-blue-700'
            }`}
          >
            {submitting ? 'Submitting...' : (existingBallot?.is_submitted ? 'Update Ballot' : 'Submit Ballot')}
          </button>
        </div>
      </main>
    </div>
  );
}
