import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import type { Session } from '@/lib/types'

function formatDuration(seconds: number | null): string {
  if (seconds == null) return '—'
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  return `${h}h ${m}m`
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

function formatDate(iso: string): string {
  const d = new Date(iso)
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

export function SessionTable({ sessions, onSelectSession }: { sessions: Session[]; onSelectSession?: (id: string) => void }) {
  if (sessions.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        No sessions found. Start an AI coding session to see data here.
      </div>
    )
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Session</TableHead>
          <TableHead>Agent</TableHead>
          <TableHead>Model</TableHead>
          <TableHead className="text-right">Turns</TableHead>
          <TableHead className="text-right">Tools</TableHead>
          <TableHead className="text-right">Tokens</TableHead>
          <TableHead className="text-right">Cost</TableHead>
          <TableHead className="text-right">Duration</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {sessions.map((s) => (
          <TableRow
            key={s.session_id}
            className={onSelectSession ? 'cursor-pointer hover:bg-accent/50' : ''}
            onClick={() => onSelectSession?.(s.session_id)}
          >
            <TableCell>
              <div className="font-mono text-xs">{s.session_id.slice(0, 20)}</div>
              <div className="text-xs text-muted-foreground">{formatDate(s.started_at)}</div>
            </TableCell>
            <TableCell>
              <Badge variant="outline">{s.agent}</Badge>
            </TableCell>
            <TableCell className="text-xs text-muted-foreground">
              {s.model ?? '—'}
            </TableCell>
            <TableCell className="text-right">{s.turns}</TableCell>
            <TableCell className="text-right">{s.tool_calls}</TableCell>
            <TableCell className="text-right font-mono text-xs">
              {formatTokens(s.tokens.input + s.tokens.output)}
            </TableCell>
            <TableCell className="text-right text-emerald-600 dark:text-emerald-400 font-mono text-xs">
              {s.est_cost_usd != null ? `$${s.est_cost_usd.toFixed(3)}` : '—'}
            </TableCell>
            <TableCell className="text-right text-xs text-muted-foreground">
              {formatDuration(s.duration_s)}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
