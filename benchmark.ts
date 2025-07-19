const urlBase =
  'http://127.0.0.1:3000/d/images/recipes/1200/grekisk-marinerad-kikartssallad-med-fetaost-och-picklad-rodlok-1.webp'

const sizes = [480, 640, 1200, 2048]
const formats = ['png', 'webp', 'avif']

async function timeRequest(fullUrl: string): Promise<number> {
  const start = performance.now()
  const res = await fetch(fullUrl)
  await res.arrayBuffer()
  const end = performance.now()
  return end - start
}

function buildUrl(params: Record<string, string | number>): string {
  const search = new URLSearchParams(params as Record<string, string>)
  return `${urlBase}?${search.toString()}`
}

async function run() {
  const results: { test: string; url: string; time: number }[] = []

  // Baseline (no params)
  const baselineTime = await timeRequest(urlBase)
  results.push({ test: 'baseline', url: urlBase, time: baselineTime })

  // Sizes only
  for (const size of sizes) {
    const testUrl = buildUrl({ size })
    const t = await timeRequest(testUrl)
    results.push({ test: `size ${size}`, url: testUrl, time: t })
  }

  // Formats only
  for (const format of formats) {
    const testUrl = buildUrl({ format })
    const t = await timeRequest(testUrl)
    results.push({ test: `format ${format}`, url: testUrl, time: t })
  }

  // Sizes + formats
  for (const size of sizes) {
    for (const format of formats) {
      const testUrl = buildUrl({ size, format })
      const t = await timeRequest(testUrl)
      results.push({
        test: `size ${size} + format ${format}`,
        url: testUrl,
        time: t
      })
    }
  }

  // Print results
  console.log('\nBenchmark Results (ms):')
  console.table(
    results.map((r) => ({
      Test: r.test,
      Time: r.time.toFixed(2)
    }))
  )
}

run().catch((err) => {
  console.error(err)

  process.exit(1)
})
