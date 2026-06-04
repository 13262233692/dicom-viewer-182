<template>
  <div class="sr-node" :style="{ paddingLeft: depth * 16 + 'px' }">
    <div
      class="sr-node-header"
      :class="{ active: node.node_id === activeNodeId }"
      @click="handleClick"
    >
      <span class="node-toggle" v-if="node.children.length > 0" @click.stop="toggleExpand">
        {{ isExpanded ? '▼' : '▶' }}
      </span>
      <span class="node-toggle leaf" v-else>•</span>

      <span class="node-type-badge" :class="node.value_type.toLowerCase()">
        {{ node.value_type }}
      </span>

      <span class="node-name">{{ node.concept_name || 'Unknown' }}</span>

      <span v-if="node.numeric_value != null" class="node-numeric">
        {{ node.numeric_value.toFixed(2) }} {{ node.numeric_unit }}
      </span>

      <span v-if="node.text_value && node.value_type === 'TEXT'" class="node-text">
        "{{ truncateText(node.text_value) }}"
      </span>

      <span v-if="node.image_reference" class="node-ref">🖼️</span>
      <span v-if="node.spatial_coordinates" class="node-scoord">📍</span>
    </div>

    <div v-if="isExpanded && node.children.length > 0" class="sr-node-children">
      <SRNodeTree
        v-for="child in node.children"
        :key="child.node_id"
        :node="child"
        :depth="depth + 1"
        :activeNodeId="activeNodeId"
        @node-click="$emit('node-click', $event)"
      />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({
  node: Object,
  depth: { type: Number, default: 0 },
  activeNodeId: String
})

const emit = defineEmits(['node-click'])

const isExpanded = ref(props.depth < 2)

const toggleExpand = () => {
  isExpanded.value = !isExpanded.value
}

const handleClick = () => {
  emit('node-click', props.node)
}

const truncateText = (text) => {
  if (text.length > 60) return text.substring(0, 60) + '...'
  return text
}
</script>

<style scoped>
.sr-node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 6px;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  transition: background 0.1s;
}

.sr-node-header:hover {
  background: rgba(100, 255, 218, 0.05);
}

.sr-node-header.active {
  background: rgba(100, 255, 218, 0.1);
  border-left: 2px solid #64ffda;
}

.node-toggle {
  font-size: 8px;
  color: #64ffda;
  width: 12px;
  text-align: center;
  flex-shrink: 0;
}

.node-toggle.leaf {
  color: #44475a;
}

.node-type-badge {
  font-size: 9px;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 600;
  text-transform: uppercase;
  flex-shrink: 0;
}

.node-type-badge.container {
  background: rgba(33, 150, 243, 0.15);
  color: #2196f3;
}

.node-type-badge.text {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
}

.node-type-badge.num {
  background: rgba(255, 193, 7, 0.15);
  color: #ffc107;
}

.node-type-badge.code {
  background: rgba(156, 39, 176, 0.15);
  color: #ce93d8;
}

.node-type-badge.image {
  background: rgba(0, 188, 212, 0.15);
  color: #00bcd4;
}

.node-type-badge.scoord {
  background: rgba(255, 152, 0, 0.15);
  color: #ff9800;
}

.node-name {
  color: #ccd6f6;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 60px;
}

.node-numeric {
  color: #ffc107;
  font-family: monospace;
  font-weight: 600;
  flex-shrink: 0;
}

.node-text {
  color: #8892b0;
  font-style: italic;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-ref, .node-scoord {
  font-size: 10px;
  flex-shrink: 0;
}

.sr-node-children {
  border-left: 1px solid #2a2a3e;
  margin-left: 10px;
}
</style>
