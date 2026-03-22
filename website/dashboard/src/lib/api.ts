import type { StatusResponse, Session, ReportResponse, EntireState, TranscriptResponse } from './types'

const BASE = '/api'

async function fetchJSON<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`API error: ${res.status}`)
  return res.json()
}

export function fetchStatus(): Promise<StatusResponse> {
  return fetchJSON(`${BASE}/status`)
}

export function fetchSessions(days = 30): Promise<Session[]> {
  return fetchJSON(`${BASE}/sessions?days=${days}`)
}

export function fetchReport(days = 30): Promise<ReportResponse> {
  return fetchJSON(`${BASE}/report?days=${days}`)
}

export function fetchEntireStates(): Promise<EntireState[]> {
  return fetchJSON(`${BASE}/entire-states`)
}

export function fetchSession(id: string): Promise<Session> {
  return fetchJSON(`${BASE}/session?id=${encodeURIComponent(id)}`)
}

export function fetchTranscript(id: string): Promise<TranscriptResponse> {
  return fetchJSON(`${BASE}/transcript?id=${encodeURIComponent(id)}`)
}
