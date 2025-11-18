<template>
  <div>
    <div class="flex justify-end mb-4 space-x-2">
      <button
        @click="importMappings"
        class="btn-secondary text-sm"
      >
        导入配置
      </button>
      <button
        @click="exportMappings"
        class="btn-secondary text-sm"
        :disabled="localMappings.length === 0"
      >
        导出配置
      </button>
    </div>

    <!-- 映射列表 -->
    <div class="space-y-3 mb-4">
      <div 
        v-for="(mapping, index) in localMappings" 
        :key="index"
        class="flex items-center space-x-3 p-3 border border-gray-200 rounded-md bg-gray-50"
      >
        <div class="flex-1">
          <input
            v-model="mapping.original"
            type="text"
            placeholder="原列名"
            class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          />
        </div>
        
        <div class="text-gray-400 font-bold">→</div>
        
        <div class="flex-1">
          <input
            v-model="mapping.mapped"
            type="text"
            placeholder="新列名"
            class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          />
        </div>
        
        <button
          @click="removeMapping(index)"
          class="text-red-500 hover:text-red-700 p-1 rounded"
          title="删除映射"
        >
          ❌
        </button>
      </div>

      <!-- 空状态 -->
      <div v-if="localMappings.length === 0" class="text-center py-8 text-gray-500">
        <div class="text-4xl mb-2">📋</div>
        <p>还没有配置列名映射</p>
        <p class="text-sm">点击下方"添加映射"按钮开始配置</p>
      </div>
    </div>

    <!-- 添加按钮 -->
    <div class="flex justify-center">
      <button
        @click="addMapping"
        class="btn-primary"
      >
        ➕ 添加映射
      </button>
    </div>

    <!-- 批量操作 -->
    <div class="mt-6 pt-4 border-t border-gray-200">
      <div class="flex justify-between items-center">
        <span class="text-sm text-gray-600">
          {{ localMappings.length > 0 ? `已配置 ${localMappings.length} 个映射` : '可以选择跳过此步骤' }}
        </span>
        <div class="space-x-2">
          <button
            v-if="localMappings.length > 0"
            @click="clearAllMappings"
            class="text-red-500 hover:text-red-700 text-sm"
          >
            清空所有
          </button>
          <button
            @click="skipMappings"
            class="btn-secondary text-sm"
          >
            跳过配置
          </button>
          <button
            v-if="localMappings.length > 0"
            @click="saveMappings"
            class="btn-primary text-sm"
          >
            保存映射
          </button>
        </div>
      </div>
    </div>

    <!-- 隐藏的文件输入 -->
    <input
      ref="fileInput"
      type="file"
      accept=".json"
      @change="handleFileImport"
      class="hidden"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useLoggerStore } from '@/stores/logger';
import type { ColumnMapping } from '@/types';

interface Props {
  modelValue: ColumnMapping[];
}

interface Emits {
  (e: 'update:modelValue', value: ColumnMapping[]): void;
  (e: 'save', value: ColumnMapping[]): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const loggerStore = useLoggerStore();
const localMappings = ref<ColumnMapping[]>([...props.modelValue]);
const fileInput = ref<HTMLInputElement>();

// 监听外部数据变化
watch(() => props.modelValue, (newValue) => {
  localMappings.value = [...newValue];
}, { deep: true });

// 监听本地数据变化，同步到外部
watch(localMappings, (newValue) => {
  emit('update:modelValue', [...newValue]);
}, { deep: true });

function addMapping() {
  localMappings.value.push({
    original: '',
    mapped: ''
  });
  loggerStore.info(`添加新的列映射，当前共 ${localMappings.value.length} 个映射`);
}

function removeMapping(index: number) {
  const mapping = localMappings.value[index];
  localMappings.value.splice(index, 1);
  loggerStore.info(`删除列映射: ${mapping.original} -> ${mapping.mapped}`);
}

function clearAllMappings() {
  if (confirm('确定要清空所有映射配置吗？')) {
    const count = localMappings.value.length;
    localMappings.value = [];
    loggerStore.warn(`已清空所有 ${count} 个列映射`);
  }
}

function saveMappings() {
  // 过滤掉空的映射
  const validMappings = localMappings.value.filter(
    mapping => mapping.original.trim() && mapping.mapped.trim()
  );
  
  loggerStore.info(`保存 ${validMappings.length} 个有效列映射`);
  validMappings.forEach(m => {
    loggerStore.info(`  映射: ${m.original} -> ${m.mapped}`);
  });
  
  emit('save', validMappings);
  loggerStore.success('列映射配置保存成功');
}

function skipMappings() {
  // 跳过配置，传递空数组
  loggerStore.info('用户选择跳过列映射配置');
  emit('save', []);
}

function importMappings() {
  loggerStore.info('打开导入映射配置对话框');
  fileInput.value?.click();
}

function handleFileImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) {
    loggerStore.warn('未选择导入文件');
    return;
  }

  loggerStore.info(`导入映射配置文件: ${file.name}`);
  
  const reader = new FileReader();
  reader.onload = (e) => {
    try {
      const content = e.target?.result as string;
      const imported = JSON.parse(content) as ColumnMapping[];
      
      if (Array.isArray(imported) && imported.every(item => 
        typeof item === 'object' && 
        typeof item.original === 'string' && 
        typeof item.mapped === 'string'
      )) {
        localMappings.value = imported;
        loggerStore.success(`成功导入 ${imported.length} 个列映射`);
      } else {
        loggerStore.error('文件格式不正确');
        alert('文件格式不正确，请选择有效的映射配置文件');
      }
    } catch (error) {
      loggerStore.error(`文件解析失败: ${error instanceof Error ? error.message : String(error)}`);
      alert('文件解析失败，请检查文件格式');
    }
  };
  
  reader.readAsText(file);
  
  // 重置文件输入
  if (fileInput.value) {
    fileInput.value.value = '';
  }
}

function exportMappings() {
  loggerStore.info(`导出 ${localMappings.value.length} 个列映射配置`);
  
  const dataStr = JSON.stringify(localMappings.value, null, 2);
  const dataBlob = new Blob([dataStr], { type: 'application/json' });
  
  const link = document.createElement('a');
  link.href = URL.createObjectURL(dataBlob);
  const fileName = `column-mappings-${new Date().getTime()}.json`;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  
  URL.revokeObjectURL(link.href);
  
  loggerStore.success(`列映射配置已导出: ${fileName}`);
}
</script>