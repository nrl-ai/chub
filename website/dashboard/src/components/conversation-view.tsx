import { useEffect, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { X, MessageSquare, Terminal, User, Bot } from 'lucide-react'
import { fetchTranscript } from '@/lib/api'
import type { ConversationMessage } from '@/lib/types'

export function ConversationView({ sessionId, onClose }: { sessionId: string; onClose: () => void }) {
  const [messages, setMessages] = useState<ConversationMessage[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    setError(null)
    fetchTranscript(sessionId)
      .then((data) => {
        setMessages(data.messages)
        if (data.error) setError(data.error)
      })
      .catch(() => setError('Failed to load transcript'))
      .finally(() => setLoading(false))
  }, [sessionId])

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <MessageSquare className="h-4 w-4" />
            <CardTitle className="text-sm font-medium">
              Conversation
            </CardTitle>
            <span className="text-xs text-muted-foreground font-mono">{sessionId.slice(0, 20)}</span>
          </div>
          <button
            onClick={onClose}
            className="inline-flex items-center justify-center rounded-md h-7 w-7 hover:bg-accent hover:text-accent-foreground transition-colors"
            title="Close"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      </CardHeader>
      <CardContent>
        {loading && (
          <div className="text-center py-8 text-muted-foreground text-sm">Loading transcript...</div>
        )}
        {error && !loading && (
          <div className="text-center py-8 text-muted-foreground text-sm">{error}</div>
        )}
        {!loading && messages.length === 0 && !error && (
          <div className="text-center py-8 text-muted-foreground text-sm">No messages found</div>
        )}
        {!loading && messages.length > 0 && (
          <div className="space-y-3 max-h-[600px] overflow-y-auto pr-1">
            {messages.map((msg, i) => (
              <MessageBubble key={i} message={msg} />
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function MessageBubble({ message }: { message: ConversationMessage }) {
  if (message.role === 'tool') {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 text-xs text-muted-foreground">
        <Terminal className="h-3 w-3 shrink-0" />
        <Badge variant="outline" className="text-[10px] px-1.5 py-0">{message.tool}</Badge>
        {message.file && (
          <span className="font-mono truncate">{message.file}</span>
        )}
      </div>
    )
  }

  const isUser = message.role === 'user'

  return (
    <div className={`flex gap-2.5 ${isUser ? '' : ''}`}>
      <div className={`shrink-0 mt-1 flex h-6 w-6 items-center justify-center rounded-full ${
        isUser ? 'bg-primary/10 text-primary' : 'bg-chart-1/10 text-chart-1'
      }`}>
        {isUser ? <User className="h-3.5 w-3.5" /> : <Bot className="h-3.5 w-3.5" />}
      </div>
      <div className="min-w-0 flex-1">
        <div className="text-[10px] font-medium text-muted-foreground mb-0.5">
          {isUser ? 'You' : 'Assistant'}
        </div>
        <div className={`text-sm leading-relaxed whitespace-pre-wrap break-words ${
          isUser ? '' : 'text-foreground/90'
        }`}>
          {message.content.length > 2000
            ? message.content.slice(0, 2000) + '...'
            : message.content}
        </div>
      </div>
    </div>
  )
}
