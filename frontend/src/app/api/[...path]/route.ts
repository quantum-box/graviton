import { NextRequest, NextResponse } from 'next/server'

const STRIP_REQUEST = new Set(['connection', 'keep-alive', 'proxy-authenticate', 'proxy-authorization', 'te', 'trailers', 'transfer-encoding', 'upgrade', 'host'])
const STRIP_RESPONSE = new Set(['connection', 'keep-alive', 'proxy-authenticate', 'proxy-authorization', 'te', 'trailers', 'transfer-encoding', 'upgrade', 'content-encoding', 'content-length'])

async function proxy(req: NextRequest, path: string[]) {
  const backendUrl = process.env.BACKEND_URL ?? 'http://localhost:8000'
  const targetUrl = new URL(`/api/${path.join('/')}`, backendUrl)
  targetUrl.search = req.nextUrl.search

  const forwardHeaders = new Headers()
  req.headers.forEach((value, key) => {
    if (!STRIP_REQUEST.has(key.toLowerCase())) forwardHeaders.set(key, value)
  })

  let upstream: Response
  try {
    upstream = await fetch(targetUrl.toString(), {
      method: req.method,
      headers: forwardHeaders,
      body: req.method === 'GET' || req.method === 'HEAD' ? undefined : req.body,
      // @ts-expect-error duplex is required in node runtime
      duplex: 'half',
    })
  } catch {
    return new NextResponse(JSON.stringify({ error: 'Backend unavailable' }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    })
  }

  const responseHeaders = new Headers()
  upstream.headers.forEach((value, key) => {
    if (!STRIP_RESPONSE.has(key.toLowerCase())) responseHeaders.set(key, value)
  })

  if (upstream.status === 304) {
    return new NextResponse(null, { status: 304, headers: responseHeaders })
  }
  return new NextResponse(upstream.body, { status: upstream.status, headers: responseHeaders })
}

export async function GET(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, (await params).path)
}
export async function POST(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, (await params).path)
}
export async function PUT(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, (await params).path)
}
export async function DELETE(req: NextRequest, { params }: { params: Promise<{ path: string[] }> }) {
  return proxy(req, (await params).path)
}
