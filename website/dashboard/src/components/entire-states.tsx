import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import type { EntireState } from '@/lib/types'

function phaseVariant(phase: string): "default" | "secondary" | "outline" | "destructive" {
  switch (phase) {
    case 'active': return 'default'
    case 'idle': return 'secondary'
    default: return 'outline'
  }
}

export function EntireStates({ states }: { states: EntireState[] }) {
  if (states.length === 0) return null

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium">entire.io Sessions</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {states.map((s) => (
            <div key={s.sessionID} className="flex items-center justify-between rounded-md border p-3">
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs">{s.sessionID.slice(0, 16)}</span>
                  <Badge variant={phaseVariant(s.phase)}>{s.phase}</Badge>
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  {s.agentType ?? 'unknown'} &middot; {s.stepCount} steps &middot; {s.filesTouched.length} files
                </div>
              </div>
              <div className="text-xs text-muted-foreground text-right">
                {new Date(s.startedAt).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
