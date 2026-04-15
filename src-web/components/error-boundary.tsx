// Author: Quadri Atharu
'use client'
import React from 'react'

interface Props {
  children: React.ReactNode
  fallback?: React.ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback
      return (
        <div style={{ padding: 24, background: '#F8F9FA', borderRadius: 8, margin: 16 }}>
          <h3 style={{ color: '#DC2626', fontFamily: 'DM Serif Display', fontSize: 18, margin: '0 0 8px 0' }}>
            Something went wrong
          </h3>
          <p style={{ color: '#1A1A2E', fontFamily: 'Inter', fontSize: 14, margin: '0 0 16px 0' }}>
            {this.state.error?.message || 'An unexpected error occurred'}
          </p>
          <button
            onClick={() => this.setState({ hasError: false, error: null })}
            style={{
              padding: '8px 16px', background: '#1B4332', color: '#FFFFFF',
              border: 'none', borderRadius: 6, fontFamily: 'Inter', fontSize: 14,
              cursor: 'pointer',
            }}
          >
            Try Again
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
