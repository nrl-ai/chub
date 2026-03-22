import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from 'recharts'
import type { ReportResponse } from '@/lib/types'

const COLORS = [
  'var(--chart-1)',
  'var(--chart-2)',
  'var(--chart-3)',
  'var(--chart-4)',
  'var(--chart-5)',
  'var(--primary)',
]

function agentToChartData(entries: [string, number, number][]): { name: string; value: number }[] {
  return entries.slice(0, 8).map(([name, sessions]) => ({ name, value: sessions }))
}

function modelToChartData(entries: [string, number, number][]): { name: string; value: number }[] {
  return entries.slice(0, 8).map(([name, sessions]) => ({ name, value: sessions }))
}

function toolToChartData(entries: [string, number][]): { name: string; value: number }[] {
  return entries.slice(0, 8).map(([name, value]) => ({ name, value }))
}

function MiniBar({ data, label }: { data: { name: string; value: number }[]; label: string }) {
  if (data.length === 0) {
    return <p className="text-sm text-muted-foreground py-4 text-center">No {label.toLowerCase()} data</p>
  }

  return (
    <ResponsiveContainer width="100%" height={data.length * 36 + 20}>
      <BarChart data={data} layout="vertical" margin={{ left: 0, right: 12, top: 4, bottom: 4 }}>
        <XAxis type="number" hide />
        <YAxis type="category" dataKey="name" width={100} tick={{ fontSize: 12, fill: 'var(--muted-foreground)' }} />
        <Tooltip
          contentStyle={{ background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 6, fontSize: 12 }}
          labelStyle={{ color: 'var(--foreground)' }}
        />
        <Bar dataKey="value" radius={[0, 4, 4, 0]}>
          {data.map((_, i) => (
            <Cell key={i} fill={COLORS[i % COLORS.length]} />
          ))}
        </Bar>
      </BarChart>
    </ResponsiveContainer>
  )
}

export function BreakdownCharts({ report }: { report: ReportResponse | null }) {
  const agentData = agentToChartData(report?.by_agent ?? [])
  const modelData = modelToChartData(report?.by_model ?? [])
  const toolData = toolToChartData(report?.top_tools ?? [])

  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">By Agent</CardTitle>
        </CardHeader>
        <CardContent>
          <MiniBar data={agentData} label="agent" />
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">By Model</CardTitle>
        </CardHeader>
        <CardContent>
          <MiniBar data={modelData} label="model" />
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Top Tools</CardTitle>
        </CardHeader>
        <CardContent>
          <MiniBar data={toolData} label="tool" />
        </CardContent>
      </Card>
    </div>
  )
}
