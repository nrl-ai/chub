import { useEffect, useState, useCallback } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { RefreshCw, Sun, Moon, Monitor } from 'lucide-react'

import { StatCards } from '@/components/stat-cards'
import { SessionTable } from '@/components/session-table'
import { BreakdownCharts } from '@/components/breakdown-charts'
import { EntireStates } from '@/components/entire-states'
import { ConversationView } from '@/components/conversation-view'
import { fetchStatus, fetchSessions, fetchReport, fetchEntireStates } from '@/lib/api'
import { useTheme } from '@/lib/theme'
import type { StatusResponse, Session, ReportResponse, EntireState } from '@/lib/types'

function App() {
  const [days, setDays] = useState(30)
  const [status, setStatus] = useState<StatusResponse | null>(null)
  const [sessions, setSessions] = useState<Session[]>([])
  const [report, setReport] = useState<ReportResponse | null>(null)
  const [entireStates, setEntireStates] = useState<EntireState[]>([])
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date())
  const [loading, setLoading] = useState(true)
  const [selectedSession, setSelectedSession] = useState<string | null>(null)
  const { theme, cycle } = useTheme()

  const refresh = useCallback(async () => {
    try {
      const [s, sess, r, e] = await Promise.all([
        fetchStatus(),
        fetchSessions(days),
        fetchReport(days),
        fetchEntireStates(),
      ])
      setStatus(s)
      setSessions(sess)
      setReport(r)
      setEntireStates(e)
      setLastRefresh(new Date())
    } catch {
      // API not available — dashboard server may not be running
    } finally {
      setLoading(false)
    }
  }, [days])

  useEffect(() => {
    refresh()
    const interval = setInterval(refresh, 10_000)
    return () => clearInterval(interval)
  }, [refresh])

  return (
    <div className="min-h-screen bg-background">
      <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">
              <span className="text-primary">Chub</span> Tracking Dashboard
            </h1>
            <p className="text-sm text-muted-foreground mt-1">
              AI agent session tracking &middot; Updated {lastRefresh.toLocaleTimeString()}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <Select value={String(days)} onValueChange={(v) => setDays(Number(v))}>
              <SelectTrigger className="w-[130px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="7">Last 7 days</SelectItem>
                <SelectItem value="14">Last 14 days</SelectItem>
                <SelectItem value="30">Last 30 days</SelectItem>
                <SelectItem value="90">Last 90 days</SelectItem>
              </SelectContent>
            </Select>
            <button
              onClick={cycle}
              className="inline-flex items-center justify-center rounded-md border border-input bg-background h-9 w-9 hover:bg-accent hover:text-accent-foreground transition-colors"
              title={`Theme: ${theme}`}
            >
              {theme === 'dark' ? <Moon className="h-4 w-4" /> : theme === 'light' ? <Sun className="h-4 w-4" /> : <Monitor className="h-4 w-4" />}
            </button>
            <button
              onClick={refresh}
              className="inline-flex items-center justify-center rounded-md border border-input bg-background h-9 w-9 hover:bg-accent hover:text-accent-foreground transition-colors"
              title="Refresh"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            </button>
          </div>
        </div>

        {/* Stats */}
        <StatCards status={status} report={report} />

        <Separator className="my-6" />

        {/* Charts */}
        <BreakdownCharts report={report} />

        <Separator className="my-6" />

        {/* Sessions table */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Session History</CardTitle>
          </CardHeader>
          <CardContent>
            <SessionTable sessions={sessions} onSelectSession={setSelectedSession} />
          </CardContent>
        </Card>

        {/* Conversation viewer */}
        {selectedSession && (
          <>
            <Separator className="my-6" />
            <ConversationView sessionId={selectedSession} onClose={() => setSelectedSession(null)} />
          </>
        )}

        {/* entire.io states */}
        {entireStates.length > 0 && (
          <>
            <Separator className="my-6" />
            <EntireStates states={entireStates} />
          </>
        )}

        {/* Footer */}
        <div className="text-center text-xs text-muted-foreground py-8">
          Chub v0.1.15 &middot; Auto-refreshes every 10s &middot; API at /api/*
        </div>
      </div>
    </div>
  )
}

export default App
