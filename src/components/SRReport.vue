<template>
  <div class="sr-report" v-if="srDocument">
    <div class="sr-header">
      <h3 class="sr-title">Structured Report</h3>
      <div class="sr-meta">
        <span v-if="srDocument.completion_flag" class="sr-badge" :class="completionClass">
          {{ srDocument.completion_flag }}
        </span>
        <span v-if="srDocument.verification_flag" class="sr-badge verify">
          {{ srDocument.verification_flag }}
        </span>
      </div>
    </div>

    <div v-if="srDocument.conclusions.length > 0" class="sr-section">
      <div class="sr-section-title" @click="toggleSection('conclusions')">
        <span class="toggle-icon">{{ expandedSections.conclusions ? '▼' : '▶' }}</span>
        Conclusions
        <span class="count-badge">{{ srDocument.conclusions.length }}</span>
      </div>
      <div v-if="expandedSections.conclusions" class="sr-section-content">
        <div
          v-for="(c, idx) in srDocument.conclusions"
          :key="'c-' + idx"
          class="conclusion-item"
          :class="{ active: activeConclusionIdx === idx }"
          @click="selectConclusion(idx)"
        >
          <div class="conclusion-text">{{ c.text || c.code_meaning }}</div>
          <div v-if="c.code_value" class="conclusion-code">{{ c.code_value }}</div>
          <div v-if="c.image_reference" class="conclusion-ref" @click.stop="navigateToImage(c.image_reference)">
            🖼️ View Image
          </div>
        </div>
      </div>
    </div>

    <div v-if="srDocument.measurements.length > 0" class="sr-section">
      <div class="sr-section-title" @click="toggleSection('measurements')">
        <span class="toggle-icon">{{ expandedSections.measurements ? '▼' : '▶' }}</span>
        Measurements
        <span class="count-badge">{{ srDocument.measurements.length }}</span>
      </div>
      <div v-if="expandedSections.measurements" class="sr-section-content">
        <div
          v-for="(m, idx) in srDocument.measurements"
          :key="'m-' + idx"
          class="measurement-item"
          :class="{ active: activeMeasurementIdx === idx }"
          @click="selectMeasurement(idx)"
        >
          <div class="measurement-name">{{ m.name }}</div>
          <div class="measurement-value">
            {{ m.value.toFixed(2) }}
            <span class="measurement-unit">{{ m.unit }}</span>
          </div>
          <div v-if="m.spatial_coordinates" class="measurement-scoord">
            📍 {{ m.spatial_coordinates.graphic_type }}
            ({{ m.spatial_coordinates.graphic_data.length }} pts)
          </div>
          <div v-if="m.image_reference" class="measurement-ref" @click.stop="navigateToImage(m.image_reference)">
            🖼️ View Image
          </div>
        </div>
      </div>
    </div>

    <div class="sr-section">
      <div class="sr-section-title" @click="toggleSection('tree')">
        <span class="toggle-icon">{{ expandedSections.tree ? '▼' : '▶' }}</span>
        Report Tree
      </div>
      <div v-if="expandedSections.tree" class="sr-section-content tree-content">
        <SRNodeTree
          :node="srDocument.root_node"
          :depth="0"
          :activeNodeId="activeNodeId"
          @node-click="handleNodeClick"
        />
      </div>
    </div>

    <div v-if="srDocument.image_references.length > 0" class="sr-section">
      <div class="sr-section-title" @click="toggleSection('refs')">
        <span class="toggle-icon">{{ expandedSections.refs ? '▼' : '▶' }}</span>
        Image References
        <span class="count-badge">{{ srDocument.image_references.length }}</span>
      </div>
      <div v-if="expandedSections.refs" class="sr-section-content">
        <div
          v-for="(r, idx) in srDocument.image_references"
          :key="'r-' + idx"
          class="ref-item"
          @click="navigateToImage(r)"
        >
          <span class="ref-uid">{{ r.sop_instance_uid.substring(0, 16) }}...</span>
          <span v-if="r.frame_number" class="ref-frame">Frame {{ r.frame_number }}</span>
        </div>
      </div>
    </div>
  </div>

  <div v-else class="sr-empty">
    <div class="sr-empty-icon">📋</div>
    <div class="sr-empty-text">No SR Report Loaded</div>
    <div class="sr-empty-hint">Open a DICOM SR file to view the structured report</div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import SRNodeTree from './SRNodeTree.vue'

const props = defineProps({
  srDocument: Object
})

const emit = defineEmits(['navigate-to-image', 'highlight-sr-item'])

const activeConclusionIdx = ref(-1)
const activeMeasurementIdx = ref(-1)
const activeNodeId = ref('')

const expandedSections = reactive({
  conclusions: true,
  measurements: true,
  tree: false,
  refs: false
})

const completionClass = ref('')

const toggleSection = (section) => {
  expandedSections[section] = !expandedSections[section]
}

const selectConclusion = (idx) => {
  activeConclusionIdx.value = idx
  activeMeasurementIdx.value = -1
  const c = props.srDocument.conclusions[idx]
  if (c.image_reference) {
    emit('navigate-to-image', c.image_reference)
  }
  emit('highlight-sr-item', { type: 'conclusion', index: idx, data: c })
}

const selectMeasurement = (idx) => {
  activeMeasurementIdx.value = idx
  activeConclusionIdx.value = -1
  const m = props.srDocument.measurements[idx]
  if (m.image_reference) {
    emit('navigate-to-image', m.image_reference)
  }
  if (m.spatial_coordinates) {
    emit('highlight-sr-item', { type: 'measurement', index: idx, data: m })
  }
}

const handleNodeClick = (node) => {
  activeNodeId.value = node.node_id
  if (node.image_reference) {
    emit('navigate-to-image', node.image_reference)
  }
  if (node.spatial_coordinates) {
    emit('highlight-sr-item', { type: 'node', data: node })
  }
}

const navigateToImage = (imageRef) => {
  emit('navigate-to-image', imageRef)
}
</script>

<style scoped>
.sr-report {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  overflow-y: auto;
  padding: 8px;
}

.sr-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 8px;
  border-bottom: 1px solid #2a2a3e;
}

.sr-title {
  font-size: 14px;
  color: #ccd6f6;
  margin: 0;
}

.sr-meta {
  display: flex;
  gap: 6px;
}

.sr-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  text-transform: uppercase;
  font-weight: 600;
}

.sr-badge.PARTIAL {
  background: rgba(255, 193, 7, 0.2);
  color: #ffc107;
}

.sr-badge.COMPLETE {
  background: rgba(76, 175, 80, 0.2);
  color: #4caf50;
}

.sr-badge.verify {
  background: rgba(33, 150, 243, 0.2);
  color: #2196f3;
}

.sr-section {
  background: rgba(42, 42, 62, 0.5);
  border-radius: 6px;
  overflow: hidden;
}

.sr-section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: #ccd6f6;
  transition: background 0.15s;
}

.sr-section-title:hover {
  background: rgba(100, 255, 218, 0.05);
}

.toggle-icon {
  font-size: 10px;
  color: #64ffda;
}

.count-badge {
  background: rgba(100, 255, 218, 0.15);
  color: #64ffda;
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 10px;
  margin-left: auto;
}

.sr-section-content {
  padding: 8px 12px;
}

.conclusion-item {
  padding: 8px 10px;
  border-radius: 4px;
  margin-bottom: 6px;
  border-left: 3px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.conclusion-item:hover {
  background: rgba(100, 255, 218, 0.05);
}

.conclusion-item.active {
  border-left-color: #64ffda;
  background: rgba(100, 255, 218, 0.1);
}

.conclusion-text {
  font-size: 12px;
  color: #ccd6f6;
  line-height: 1.4;
}

.conclusion-code {
  font-size: 10px;
  color: #8892b0;
  margin-top: 4px;
}

.conclusion-ref {
  font-size: 11px;
  color: #64ffda;
  cursor: pointer;
  margin-top: 4px;
}

.measurement-item {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 4px;
  margin-bottom: 4px;
  border-left: 3px solid transparent;
  cursor: pointer;
  transition: all 0.15s;
}

.measurement-item:hover {
  background: rgba(100, 255, 218, 0.05);
}

.measurement-item.active {
  border-left-color: #ffc107;
  background: rgba(255, 193, 7, 0.08);
}

.measurement-name {
  font-size: 12px;
  color: #ccd6f6;
  flex: 1;
  min-width: 100px;
}

.measurement-value {
  font-size: 13px;
  font-weight: 600;
  color: #64ffda;
  font-family: monospace;
}

.measurement-unit {
  font-size: 10px;
  color: #8892b0;
  font-weight: 400;
}

.measurement-scoord {
  font-size: 10px;
  color: #ffc107;
  width: 100%;
}

.measurement-ref {
  font-size: 11px;
  color: #64ffda;
  cursor: pointer;
}

.tree-content {
  max-height: 300px;
  overflow-y: auto;
}

.ref-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}

.ref-item:hover {
  background: rgba(100, 255, 218, 0.05);
}

.ref-uid {
  font-size: 11px;
  color: #8892b0;
  font-family: monospace;
}

.ref-frame {
  font-size: 11px;
  color: #ffc107;
}

.sr-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #8892b0;
}

.sr-empty-icon {
  font-size: 32px;
  margin-bottom: 12px;
}

.sr-empty-text {
  font-size: 14px;
  color: #ccd6f6;
}

.sr-empty-hint {
  font-size: 12px;
  margin-top: 6px;
}
</style>
