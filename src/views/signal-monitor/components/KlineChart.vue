<script setup lang="ts">
import { computed } from 'vue'
import { use } from 'echarts/core'
import { CandlestickChart, LineChart, BarChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  DataZoomComponent,
  MarkLineComponent,
  MarkPointComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import VChart from 'vue-echarts'
import type { KlineData } from '@/stores/market'

use([
  CandlestickChart, LineChart, BarChart,
  GridComponent, TooltipComponent, LegendComponent,
  DataZoomComponent, MarkLineComponent, MarkPointComponent,
  CanvasRenderer,
])

const props = defineProps<{
  data: KlineData[]
  loading?: boolean
  title?: string
  signalPoints?: Array<{ index: number; type: 'buy' | 'sell'; label: string }>
}>()

const chartOption = computed(() => {
  if (!props.data || props.data.length === 0) {
    return {
      backgroundColor: 'transparent',
      title: { text: '暂无数据', left: 'center', top: 'center', textStyle: { color: '#5c6a7a', fontSize: 14 } },
    }
  }

  const timestamps = props.data.map(d => d.timestamp)
  const ohlc = props.data.map(d => [d.open, d.close, d.low, d.high])
  const volumes = props.data.map(d => d.volume)

  const ma5 = calcMA(5)
  const ma10 = calcMA(10)
  const ma20 = calcMA(20)

  const markPoints: Record<string, unknown> = {}
  if (props.signalPoints && props.signalPoints.length > 0) {
    const buyPoints: Array<{ coord: [number, number]; name: string }> = []
    const sellPoints: Array<{ coord: [number, number]; name: string }> = []
    for (const sp of props.signalPoints) {
      if (sp.index < props.data.length) {
        const d = props.data[sp.index]
        if (sp.type === 'buy') {
          buyPoints.push({ coord: [sp.index, d.low], name: sp.label })
        } else {
          sellPoints.push({ coord: [sp.index, d.high], name: sp.label })
        }
      }
    }
    if (buyPoints.length > 0 || sellPoints.length > 0) {
      markPoints.data = [
        ...buyPoints.map(p => ({
          coord: p.coord,
          name: p.name,
          symbol: 'triangle',
          symbolSize: 12,
          itemStyle: { color: '#e03f3c' },
          label: { show: true, formatter: p.name, position: 'bottom', color: '#e03f3c', fontSize: 10 },
        })),
        ...sellPoints.map(p => ({
          coord: p.coord,
          name: p.name,
          symbol: 'path://M-6,-6L6,6M6,-6L-6,6',
          symbolSize: 12,
          itemStyle: { color: '#22bc53' },
          label: { show: true, formatter: p.name, position: 'top', color: '#22bc53', fontSize: 10 },
        })),
      ]
    }
  }

  return {
    backgroundColor: 'transparent',
    animation: true,
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'cross' },
      backgroundColor: '#1a2433',
      borderColor: '#2a3a4e',
      textStyle: { color: '#e8eaed', fontSize: 12 },
      formatter(params: any) {
        const kline = params.find((p: any) => p.seriesName === 'K线')
        const vol = params.find((p: any) => p.seriesName === '成交量')
        if (!kline) return ''
        const d = kline.data as number[]
        const color = d[1] >= d[0] ? '#e03f3c' : '#22bc53'
        // 根据价格大小自动调整小数位
        const fmt = (v: number) => {
          if (v >= 10000) return v.toFixed(0)
          if (v >= 100) return v.toFixed(2)
          return v.toFixed(3)
        }
        const fmtVol = (v: any) => {
          const n = typeof v === 'object' ? v.value : v
          if (n >= 100000000) return `${(n / 100000000).toFixed(2)}亿`
          if (n >= 10000) return `${(n / 10000).toFixed(0)}万`
          return String(Math.round(n))
        }
        return `<div style="font-size:12px;line-height:1.6">
          <div>${kline.axisValue}</div>
          <div>开盘 <span style="color:${color}">${fmt(d[0])}</span></div>
          <div>收盘 <span style="color:${color}">${fmt(d[1])}</span></div>
          <div>最低 <span style="color:#22bc53">${fmt(d[2])}</span></div>
          <div>最高 <span style="color:#e03f3c">${fmt(d[3])}</span></div>
          ${vol ? `<div>成交量 ${fmtVol(vol.data)}</div>` : ''}
        </div>`
      },
    },
    legend: {
      data: ['K线', 'MA5均线', 'MA10均线', 'MA20均线'],
      top: 0,
      textStyle: { color: '#9aa5b4', fontSize: 11 },
    },
    grid: [
      { left: 60, right: 20, top: 35, height: '60%' },
      { left: 60, right: 20, top: '72%', height: '18%' },
    ],
    xAxis: [
      {
        type: 'category',
        data: timestamps,
        axisLine: { lineStyle: { color: '#1e2a3a' } },
        axisLabel: { color: '#5c6a7a', fontSize: 10 },
        splitLine: { show: false },
        gridIndex: 0,
      },
      {
        type: 'category',
        data: timestamps,
        gridIndex: 1,
        axisLabel: { show: false },
        axisLine: { lineStyle: { color: '#1e2a3a' } },
      },
    ],
    yAxis: [
      {
        type: 'value',
        scale: true,
        axisLine: { lineStyle: { color: '#1e2a3a' } },
        axisLabel: {
          color: '#5c6a7a',
          fontSize: 10,
          formatter(val: number) {
            if (val >= 10000) return String(Math.round(val))
            if (val >= 100) return val.toFixed(2)
            return val.toFixed(3)
          },
        },
        splitLine: { lineStyle: { color: '#1e2a3a', type: 'dashed' } },
        gridIndex: 0,
      },
      {
        type: 'value',
        scale: true,
        gridIndex: 1,
        axisLabel: { show: false },
        splitLine: { show: false },
      },
    ],
    dataZoom: [
      { type: 'inside', xAxisIndex: [0, 1], start: 50, end: 100 },
      { type: 'slider', xAxisIndex: [0, 1], bottom: 2, height: 14, borderColor: '#1e2a3a', fillerColor: 'rgba(206,164,62,0.15)', textStyle: { color: '#5c6a7a', fontSize: 10 } },
    ],
    series: [
      {
        name: 'K线',
        type: 'candlestick',
        data: ohlc,
        xAxisIndex: 0,
        yAxisIndex: 0,
        itemStyle: {
          color: '#e03f3c',
          color0: '#22bc53',
          borderColor: '#e03f3c',
          borderColor0: '#22bc53',
        },
        markPoint: Object.keys(markPoints).length > 0 ? markPoints : undefined,
      },
      {
        name: 'MA5均线',
        type: 'line',
        data: ma5,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1, color: '#e8a735' },
        xAxisIndex: 0,
        yAxisIndex: 0,
      },
      {
        name: 'MA10均线',
        type: 'line',
        data: ma10,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1, color: '#3b82f6' },
        xAxisIndex: 0,
        yAxisIndex: 0,
      },
      {
        name: 'MA20均线',
        type: 'line',
        data: ma20,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1, color: '#9333ea' },
        xAxisIndex: 0,
        yAxisIndex: 0,
      },
      {
        name: '成交量',
        type: 'bar',
        data: volumes.map((v, i) => ({
          value: v,
          itemStyle: {
            color: i > 0 && props.data[i].close >= props.data[i].open ? '#e03f3c' : '#22bc53',
            opacity: 0.6,
          },
        })),
        xAxisIndex: 1,
        yAxisIndex: 1,
      },
    ],
  }
})

function calcMA(dayCount: number): (number | null)[] {
  const result: (number | null)[] = []
  for (let i = 0; i < props.data.length; i++) {
    if (i < dayCount - 1) {
      result.push(null)
      continue
    }
    let sum = 0
    for (let j = 0; j < dayCount; j++) {
      sum += props.data[i - j].close
    }
    result.push(Number((sum / dayCount).toFixed(2)))
  }
  return result
}
</script>

<template>
  <div class="relative w-full h-full min-h-[400px]">
    <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-card/80 z-10">
      <span class="text-sm text-muted-foreground animate-pulse">加载K线数据...</span>
    </div>
    <VChart :option="chartOption" autoresize class="w-full h-full" />
  </div>
</template>
