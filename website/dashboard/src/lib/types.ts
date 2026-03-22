export interface TokenUsage {
  input: number
  output: number
  cache_read: number
  cache_write: number
}

export interface Session {
  session_id: string
  agent: string
  model: string | null
  started_at: string
  ended_at: string | null
  duration_s: number | null
  turns: number
  tokens: TokenUsage
  tool_calls: number
  tools_used: string[]
  files_changed: string[]
  commits: string[]
  est_cost_usd: number | null
}

export interface StatusResponse {
  active_session: {
    session_id: string
    agent: string
    model: string | null
    started_at: string
    turns: number
    tool_calls: number
  } | null
  agent_detected: string
  model_detected: string | null
  entire_sessions: number
}

// Each entry is [name, session_count, cost_or_tokens]
export type AgentEntry = [string, number, number]
export type ModelEntry = [string, number, number]
export type ToolEntry = [string, number]

export interface ReportResponse {
  period_days: number
  session_count: number
  total_duration_s: number
  total_est_cost_usd: number
  total_tokens: {
    input: number
    output: number
    cache_read: number
    cache_write: number
  }
  total_tool_calls: number
  by_agent: AgentEntry[]
  by_model: ModelEntry[]
  top_tools: ToolEntry[]
}

export interface ConversationMessage {
  role: 'user' | 'assistant' | 'tool'
  content: string
  tool?: string
  file?: string
}

export interface TranscriptResponse {
  session_id: string
  messages: ConversationMessage[]
  error?: string
}

export interface EntireState {
  sessionID: string
  phase: string
  agentType: string | null
  startedAt: string
  stepCount: number
  filesTouched: string[]
  transcriptPath: string | null
}
