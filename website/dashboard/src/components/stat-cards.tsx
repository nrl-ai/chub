import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Activity, DollarSign, Zap, Hash } from 'lucide-react'
import type { StatusResponse, ReportResponse } from '@/lib/types'

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

export function StatCards({ status, report }: { status: StatusResponse | null; report: ReportResponse | null }) {
  const totalTokens = report
    ? report.total_tokens.input + report.total_tokens.output + report.total_tokens.cache_read + report.total_tokens.cache_write
    : 0

  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Active Session</CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          {status?.active_session ? (
            <>
              <Badge variant="default" className="mb-1 bg-emerald-600 hover:bg-emerald-700">Live</Badge>
              <p className="text-xs text-muted-foreground mt-1">
                {status.active_session.agent} &middot; {status.active_session.turns} turns
              </p>
            </>
          ) : (
            <>
              <div className="text-2xl font-bold text-muted-foreground">None</div>
              <p className="text-xs text-muted-foreground">
                {status?.agent_detected !== 'unknown' ? `Detected: ${status?.agent_detected}` : 'No agent detected'}
              </p>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Total Sessions</CardTitle>
          <Hash className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{report?.session_count ?? 0}</div>
          <p className="text-xs text-muted-foreground">
            Last {report?.period_days ?? 30} days
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Estimated Cost</CardTitle>
          <DollarSign className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">
            ${(report?.total_est_cost_usd ?? 0).toFixed(2)}
          </div>
          <p className="text-xs text-muted-foreground">
            Last {report?.period_days ?? 30} days
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Total Tokens</CardTitle>
          <Zap className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{formatTokens(totalTokens)}</div>
          <p className="text-xs text-muted-foreground">
            {formatTokens(report?.total_tokens.input ?? 0)} in &middot; {formatTokens(report?.total_tokens.output ?? 0)} out
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
