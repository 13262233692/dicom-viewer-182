<template>
  <div class="viewport" ref="viewportRef">
    <canvas ref="canvasRef" class="cornerstone-canvas"></canvas>
    <div v-if="imageData" class="info-overlay">
      <div class="info-row">
        <span class="info-label">WC/WW:</span>
        <span class="info-value">{{ windowCenter }}/{{ windowWidth }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Zoom:</span>
        <span class="info-value">{{ scale.toFixed(2) }}x</span>
      </div>
    </div>
    <div v-else class="empty-state">
      <div class="empty-state-icon">🩻</div>
      <div class="empty-state-text">No DICOM Image Loaded</div>
      <div class="empty-state-hint">Open a DICOM file or directory to start</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps({
  imageData: Object,
  activeTool: String,
  windowCenter: Number,
  windowWidth: Number,
  srHighlights: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['measurement-added'])

const viewportRef = ref(null)
const canvasRef = ref(null)
const ctx = ref(null)

const scale = ref(1)
const translation = ref({ x: 0, y: 0 })
const measurements = ref([])
const isDrawing = ref(false)
const currentMeasurement = ref(null)
const lastMousePos = ref({ x: 0, y: 0 })
const pixelSpacing = ref(1)

const resizeCanvas = () => {
  if (!canvasRef.value || !viewportRef.value) return
  const rect = viewportRef.value.getBoundingClientRect()
  canvasRef.value.width = rect.width
  canvasRef.value.height = rect.height
  render()
}

const applyWindow = (value) => {
  const min = props.windowCenter - props.windowWidth / 2
  const max = props.windowCenter + props.windowWidth / 2
  if (value <= min) return 0
  if (value >= max) return 255
  return ((value - min) / props.windowWidth) * 255
}

const render = () => {
  if (!ctx.value || !canvasRef.value || !props.imageData) return
  
  const { width, height, pixel_data } = props.imageData
  const canvas = canvasRef.value
  
  ctx.value.fillStyle = '#000'
  ctx.value.fillRect(0, 0, canvas.width, canvas.height)
  
  const imageAspect = width / height
  const canvasAspect = canvas.width / canvas.height
  
  let renderWidth, renderHeight
  
  if (imageAspect > canvasAspect) {
    renderWidth = canvas.width * scale.value
    renderHeight = renderWidth / imageAspect
  } else {
    renderHeight = canvas.height * scale.value
    renderWidth = renderHeight * imageAspect
  }
  
  const offsetX = (canvas.width - renderWidth) / 2 + translation.value.x
  const offsetY = (canvas.height - renderHeight) / 2 + translation.value.y
  
  const imageData = ctx.value.createImageData(renderWidth, renderHeight)
  const data = imageData.data
  
  for (let y = 0; y < renderHeight; y++) {
    for (let x = 0; x < renderWidth; x++) {
      const srcX = Math.floor((x / renderWidth) * width)
      const srcY = Math.floor((y / renderHeight) * height)
      const srcIdx = srcY * width + srcX
      
      let value = pixel_data[srcIdx] || 0
      if (props.imageData.slope && props.imageData.intercept) {
        value = value * props.imageData.slope + props.imageData.intercept
      }
      
      const pixel = applyWindow(value)
      const dstIdx = (y * renderWidth + x) * 4
      
      data[dstIdx] = pixel
      data[dstIdx + 1] = pixel
      data[dstIdx + 2] = pixel
      data[dstIdx + 3] = 255
    }
  }
  
  ctx.value.putImageData(imageData, offsetX, offsetY)
  renderMeasurements(offsetX, offsetY, renderWidth, renderHeight)
  renderSRHighlights(offsetX, offsetY, renderWidth, renderHeight)
}

const renderMeasurements = (offsetX, offsetY, renderWidth, renderHeight) => {
  if (!ctx.value || !props.imageData) return
  
  const { width, height } = props.imageData
  pixelSpacing.value = props.imageData.spacing?.[0] || 1
  
  ctx.value.strokeStyle = '#e94560'
  ctx.value.lineWidth = 2
  ctx.value.fillStyle = '#e94560'
  ctx.value.font = '12px monospace'
  
  const allMeasurements = [...measurements.value]
  if (currentMeasurement.value) {
    allMeasurements.push(currentMeasurement.value)
  }
  
  for (const m of allMeasurements) {
    const toCanvas = (point) => ({
      x: offsetX + (point.x / width) * renderWidth,
      y: offsetY + (point.y / height) * renderHeight
    })
    
    if (m.type === 'Length' && m.points.length >= 2) {
      const p1 = toCanvas(m.points[0])
      const p2 = toCanvas(m.points[1])
      
      ctx.value.beginPath()
      ctx.value.moveTo(p1.x, p1.y)
      ctx.value.lineTo(p2.x, p2.y)
      ctx.value.stroke()
      
      ctx.value.beginPath()
      ctx.value.arc(p1.x, p1.y, 4, 0, Math.PI * 2)
      ctx.value.arc(p2.x, p2.y, 4, 0, Math.PI * 2)
      ctx.value.fill()
      
      const dist = Math.sqrt(
        Math.pow((m.points[1].x - m.points[0].x) * pixelSpacing.value, 2) +
        Math.pow((m.points[1].y - m.points[0].y) * pixelSpacing.value, 2)
      )
      
      const midX = (p1.x + p2.x) / 2
      const midY = (p1.y + p2.y) / 2
      ctx.value.fillText(`${dist.toFixed(2)} mm`, midX + 8, midY - 8)
      
    } else if (m.type === 'Angle' && m.points.length >= 3) {
      const p1 = toCanvas(m.points[0])
      const p2 = toCanvas(m.points[1])
      const p3 = toCanvas(m.points[2])
      
      ctx.value.beginPath()
      ctx.value.moveTo(p1.x, p1.y)
      ctx.value.lineTo(p2.x, p2.y)
      ctx.value.lineTo(p3.x, p3.y)
      ctx.value.stroke()
      
      ctx.value.beginPath()
      ctx.value.arc(p1.x, p1.y, 4, 0, Math.PI * 2)
      ctx.value.arc(p2.x, p2.y, 4, 0, Math.PI * 2)
      ctx.value.arc(p3.x, p3.y, 4, 0, Math.PI * 2)
      ctx.value.fill()
      
      const angle = calculateAngle(m.points[0], m.points[1], m.points[2])
      ctx.value.fillText(`${angle.toFixed(1)}°`, p2.x + 8, p2.y - 8)
      
    } else if (m.type === 'Area' && m.points.length >= 3) {
      ctx.value.beginPath()
      for (let i = 0; i < m.points.length; i++) {
        const p = toCanvas(m.points[i])
        if (i === 0) ctx.value.moveTo(p.x, p.y)
        else ctx.value.lineTo(p.x, p.y)
      }
      ctx.value.closePath()
      ctx.value.stroke()
      ctx.value.fillStyle = 'rgba(233, 69, 96, 0.2)'
      ctx.value.fill()
      ctx.value.fillStyle = '#e94560'
      
      for (const point of m.points) {
        const p = toCanvas(point)
        ctx.value.beginPath()
        ctx.value.arc(p.x, p.y, 4, 0, Math.PI * 2)
        ctx.value.fill()
      }
      
      const area = calculateArea(m.points)
      const center = m.points.reduce((acc, p) => ({ x: acc.x + p.x, y: acc.y + p.y }), { x: 0, y: 0 })
      center.x /= m.points.length
      center.y /= m.points.length
      const c = toCanvas(center)
      ctx.value.fillText(`${area.toFixed(2)} mm²`, c.x, c.y)
    }
  }
}

const calculateAngle = (p1, p2, p3) => {
  const v1 = { x: p1.x - p2.x, y: p1.y - p2.y }
  const v2 = { x: p3.x - p2.x, y: p3.y - p2.y }
  const dot = v1.x * v2.x + v1.y * v2.y
  const mag1 = Math.sqrt(v1.x * v1.x + v1.y * v1.y)
  const mag2 = Math.sqrt(v2.x * v2.x + v2.y * v2.y)
  return Math.acos(dot / (mag1 * mag2)) * (180 / Math.PI)
}

const calculateArea = (points) => {
  let area = 0
  const n = points.length
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n
    area += points[i].x * points[j].y * pixelSpacing.value * pixelSpacing.value
    area -= points[j].x * points[i].y * pixelSpacing.value * pixelSpacing.value
  }
  return Math.abs(area / 2)
}

const renderSRHighlights = (offsetX, offsetY, renderWidth, renderHeight) => {
  if (!ctx.value || !props.imageData || !props.srHighlights || props.srHighlights.length === 0) return

  const { width, height } = props.imageData

  const toCanvas = (point) => ({
    x: offsetX + (point[0] / width) * renderWidth,
    y: offsetY + (point[1] / height) * renderHeight
  })

  for (const highlight of props.srHighlights) {
    if (!highlight.spatial_coordinates) continue

    const { graphic_type, graphic_data } = highlight.spatial_coordinates
    if (!graphic_data || graphic_data.length === 0) continue

    ctx.value.save()

    const pulsePhase = (Date.now() % 2000) / 2000
    const alpha = 0.5 + 0.3 * Math.sin(pulsePhase * Math.PI * 2)

    ctx.value.strokeStyle = `rgba(0, 255, 200, ${alpha})`
    ctx.value.lineWidth = 3
    ctx.value.shadowColor = 'rgba(0, 255, 200, 0.6)'
    ctx.value.shadowBlur = 8

    if (graphic_type === 'POINT' || graphic_type === 'CIRCLE') {
      const center = toCanvas(graphic_data[0])
      if (graphic_type === 'POINT') {
        ctx.value.beginPath()
        ctx.value.arc(center.x, center.y, 8, 0, Math.PI * 2)
        ctx.value.stroke()
        ctx.value.beginPath()
        ctx.value.arc(center.x, center.y, 3, 0, Math.PI * 2)
        ctx.value.fillStyle = `rgba(0, 255, 200, ${alpha})`
        ctx.value.fill()
      } else if (graphic_data.length >= 2) {
        const radius = Math.sqrt(
          Math.pow((graphic_data[1][0] - graphic_data[0][0]) / width * renderWidth, 2) +
          Math.pow((graphic_data[1][1] - graphic_data[0][1]) / height * renderHeight, 2)
        )
        ctx.value.beginPath()
        ctx.value.arc(center.x, center.y, radius, 0, Math.PI * 2)
        ctx.value.stroke()
      }
    } else if (graphic_type === 'POLYLINE' || graphic_type === 'POLYGON') {
      ctx.value.beginPath()
      for (let i = 0; i < graphic_data.length; i++) {
        const p = toCanvas(graphic_data[i])
        if (i === 0) ctx.value.moveTo(p.x, p.y)
        else ctx.value.lineTo(p.x, p.y)
      }
      if (graphic_type === 'POLYGON') {
        ctx.value.closePath()
        ctx.value.fillStyle = `rgba(0, 255, 200, 0.15)`
        ctx.value.fill()
      }
      ctx.value.stroke()
    } else if (graphic_type === 'ELLIPSE') {
      if (graphic_data.length >= 2) {
        const center = toCanvas(graphic_data[0])
        const semiAxis = toCanvas(graphic_data[1])
        const rx = Math.abs(semiAxis.x - center.x)
        const ry = Math.abs(semiAxis.y - center.y)
        ctx.value.beginPath()
        ctx.value.ellipse(center.x, center.y, rx, ry, 0, 0, Math.PI * 2)
        ctx.value.stroke()
        ctx.value.fillStyle = `rgba(0, 255, 200, 0.15)`
        ctx.value.fill()
      }
    } else if (graphic_type === 'MULTIPOINT') {
      for (const point of graphic_data) {
        const p = toCanvas(point)
        ctx.value.beginPath()
        ctx.value.arc(p.x, p.y, 5, 0, Math.PI * 2)
        ctx.value.stroke()
      }
    }

    if (highlight.name) {
      let labelPoint
      if (graphic_data.length > 0) {
        labelPoint = toCanvas(graphic_data[0])
      }
      if (labelPoint) {
        ctx.value.shadowBlur = 0
        ctx.value.font = '11px monospace'
        ctx.value.fillStyle = `rgba(0, 255, 200, ${Math.max(alpha, 0.8)})`
        ctx.value.fillText(highlight.name, labelPoint.x + 12, labelPoint.y - 8)
      }
    }

    ctx.value.restore()
  }
}

const getImageCoords = (e) => {
  if (!canvasRef.value || !props.imageData) return null
  
  const rect = canvasRef.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top
  
  const { width, height } = props.imageData
  const canvas = canvasRef.value
  const imageAspect = width / height
  const canvasAspect = canvas.width / canvas.height
  
  let renderWidth, renderHeight
  if (imageAspect > canvasAspect) {
    renderWidth = canvas.width * scale.value
    renderHeight = renderWidth / imageAspect
  } else {
    renderHeight = canvas.height * scale.value
    renderWidth = renderHeight * imageAspect
  }
  
  const offsetX = (canvas.width - renderWidth) / 2 + translation.value.x
  const offsetY = (canvas.height - renderHeight) / 2 + translation.value.y
  
  return {
    x: ((x - offsetX) / renderWidth) * width,
    y: ((y - offsetY) / renderHeight) * height
  }
}

const handleMouseDown = (e) => {
  if (!props.imageData) return
  
  lastMousePos.value = { x: e.clientX, y: e.clientY }
  
  if (props.activeTool === 'length' || props.activeTool === 'angle' || props.activeTool === 'area') {
    const coords = getImageCoords(e)
    if (!coords) return
    
    isDrawing.value = true
    
    if (!currentMeasurement.value || props.activeTool !== currentMeasurement.value.type) {
      currentMeasurement.value = {
        type: props.activeTool === 'length' ? 'Length' : props.activeTool === 'angle' ? 'Angle' : 'Area',
        points: [coords]
      }
    } else {
      currentMeasurement.value.points.push(coords)
      
      const maxPoints = props.activeTool === 'length' ? 2 : props.activeTool === 'angle' ? 3 : Infinity
      if (currentMeasurement.value.points.length >= maxPoints) {
        finishMeasurement()
      }
    }
    render()
  } else {
    isDrawing.value = true
  }
}

const handleMouseMove = (e) => {
  if (!props.imageData) return
  
  if (props.activeTool === 'window' && isDrawing.value) {
    const dx = e.clientX - lastMousePos.value.x
    const dy = e.clientY - lastMousePos.value.y
    
    emit('window-change', 
      props.windowCenter + dy * 0.5,
      props.windowWidth + dx * 0.5
    )
    
    lastMousePos.value = { x: e.clientX, y: e.clientY }
  } else if (props.activeTool === 'zoom' && isDrawing.value) {
    const dy = e.clientY - lastMousePos.value.y
    scale.value = Math.max(0.1, Math.min(10, scale.value * (1 - dy * 0.005)))
    lastMousePos.value = { x: e.clientX, y: e.clientY }
    render()
  } else if (props.activeTool === 'pan' && isDrawing.value) {
    translation.value.x += e.clientX - lastMousePos.value.x
    translation.value.y += e.clientY - lastMousePos.value.y
    lastMousePos.value = { x: e.clientX, y: e.clientY }
    render()
  } else if (isDrawing.value && currentMeasurement.value) {
    const coords = getImageCoords(e)
    if (coords && currentMeasurement.value.points.length > 0) {
      if (currentMeasurement.value.points.length === 1) {
        currentMeasurement.value.points[1] = coords
      } else if (currentMeasurement.value.points.length >= 2) {
        currentMeasurement.value.points[currentMeasurement.value.points.length - 1] = coords
      }
      render()
    }
  }
}

const handleMouseUp = () => {
  isDrawing.value = false
}

const handleDoubleClick = () => {
  if (currentMeasurement.value && currentMeasurement.value.type === 'Area') {
    finishMeasurement()
  }
}

const finishMeasurement = () => {
  if (currentMeasurement.value) {
    let value = 0
    if (currentMeasurement.value.type === 'Length') {
      const [p1, p2] = currentMeasurement.value.points
      value = Math.sqrt(
        Math.pow((p2.x - p1.x) * pixelSpacing.value, 2) +
        Math.pow((p2.y - p1.y) * pixelSpacing.value, 2)
      )
    } else if (currentMeasurement.value.type === 'Angle') {
      value = calculateAngle(
        currentMeasurement.value.points[0],
        currentMeasurement.value.points[1],
        currentMeasurement.value.points[2]
      )
    } else if (currentMeasurement.value.type === 'Area') {
      value = calculateArea(currentMeasurement.value.points)
    }
    
    emit('measurement-added', {
      type: currentMeasurement.value.type,
      value
    })
    
    measurements.value.push(currentMeasurement.value)
    currentMeasurement.value = null
    render()
  }
}

const handleWheel = (e) => {
  if (!props.imageData) return
  e.preventDefault()
  scale.value = Math.max(0.1, Math.min(10, scale.value * (1 - e.deltaY * 0.001)))
  render()
}

const clearMeasurements = () => {
  measurements.value = []
  currentMeasurement.value = null
  render()
}

defineExpose({ clearMeasurements })

watch(() => props.imageData, () => {
  scale.value = 1
  translation.value = { x: 0, y: 0 }
  render()
})

watch([() => props.windowCenter, () => props.windowWidth], () => {
  render()
})

watch(() => props.srHighlights, () => {
  render()
}, { deep: true })

let animationFrame = null
const startSRAnimation = () => {
  if (props.srHighlights && props.srHighlights.length > 0) {
    const animate = () => {
      render()
      animationFrame = requestAnimationFrame(animate)
    }
    animationFrame = requestAnimationFrame(animate)
  }
}

const stopSRAnimation = () => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame)
    animationFrame = null
  }
}

watch(() => props.srHighlights, (newVal) => {
  stopSRAnimation()
  if (newVal && newVal.length > 0) {
    startSRAnimation()
  }
}, { deep: true })

onMounted(() => {
  ctx.value = canvasRef.value?.getContext('2d')
  window.addEventListener('resize', resizeCanvas)
  resizeCanvas()
  if (props.srHighlights && props.srHighlights.length > 0) {
    startSRAnimation()
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', resizeCanvas)
  stopSRAnimation()
})
</script>

<style scoped>
.viewport {
  width: 100%;
  height: 100%;
  position: relative;
  background: #000;
  cursor: crosshair;
}
</style>
