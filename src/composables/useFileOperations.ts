import { ref, computed, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ScanResult, ColumnMapping } from '@/types';
import { useLoggerStore } from '@/stores/logger';

export function useFileOperations() {
  const loggerStore = useLoggerStore();
  
  const sourcePath = ref('');
  const targetPath = ref('');
  const scanResult = ref<ScanResult | null>(null);
  const convertedFiles = ref<File[]>([]);
  const scanning = ref(false);
  const converting = ref(false);
  
  const canConvert = computed(() => 
    scanResult.value && scanResult.value.file_count > 0 && targetPath.value
  );
  
  const canUpload = computed(() => 
    convertedFiles.value.length > 0
  );
  
  async function selectSourceFolder() {
    try {
      loggerStore.info('正在打开文件夹选择对话框...');
      const selected = await open({
        directory: true,
        title: '选择源文件夹'
      });
      if (selected) {
        sourcePath.value = selected as string;
        loggerStore.success(`已选择源文件夹: ${sourcePath.value}`);
      } else {
        loggerStore.warn('未选择源文件夹');
      }
    } catch (error) {
      const errorMsg = `选择文件夹失败: ${error instanceof Error ? error.message : String(error)}`;
      loggerStore.error(errorMsg);
      console.error('选择文件夹失败:', error);
      throw new Error(errorMsg);
    }
  }
  
  async function selectTargetFolder() {
    try {
      loggerStore.info('正在打开目标文件夹选择对话框...');
      const selected = await open({
        directory: true,
        title: '选择目标文件夹'
      });
      if (selected) {
        targetPath.value = selected as string;
        loggerStore.success(`已选择目标文件夹: ${targetPath.value}`);
      } else {
        loggerStore.warn('未选择目标文件夹');
      }
    } catch (error) {
      const errorMsg = `选择文件夹失败: ${error instanceof Error ? error.message : String(error)}`;
      loggerStore.error(errorMsg);
      console.error('选择文件夹失败:', error);
      throw new Error(errorMsg);
    }
  }
  
  async function scanFiles() {
    if (!sourcePath.value) {
      loggerStore.warn('请先选择源文件夹');
      return;
    }
    
    scanning.value = true;
    loggerStore.info(`开始扫描文件夹: ${sourcePath.value}`);
    
    try {
      const result = await invoke<ScanResult>('scan_excel_files', {
        sourcePath: sourcePath.value
      });
      scanResult.value = result;
      loggerStore.success(`扫描完成，发现 ${result.file_count} 个Excel文件，总大小: ${(result.total_size / 1024 / 1024).toFixed(2)} MB`);
    } catch (error) {
      const errorMsg = `扫描文件失败: ${error instanceof Error ? error.message : String(error)}`;
      loggerStore.error(errorMsg);
      console.error('扫描文件失败:', error);
      throw new Error(errorMsg);
    } finally {
      scanning.value = false;
    }
  }
  
  async function startConversion(mappings: ColumnMapping[]) {
    if (!canConvert.value) {
      loggerStore.warn('请先扫描文件并选择目标文件夹');
      return;
    }
    
    converting.value = true;
    loggerStore.info(`开始转换文件，使用 ${mappings.length} 个列映射规则`);
    
    try {
      const convertedFilePaths = await invoke<string[]>('convert_excel_files', {
        sourcePath: sourcePath.value,
        targetPath: targetPath.value,
        mappings
      });
      
      loggerStore.info(`文件转换完成，共 ${convertedFilePaths.length} 个文件`);
      
      // 将文件路径转换为File对象
      const files = await Promise.all(
        convertedFilePaths.map(async (filePath) => {
          // 在 Tauri 环境中，我们需要使用特殊的方式来读取文件
          const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'unknown.xlsx';
          
          // 创建一个虚拟的 File 对象，实际文件读取在上传时进行
          const file = new File([], fileName, { 
            type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' 
          });
          
          // 添加文件路径属性，用于后续读取
          (file as any).filePath = filePath;
          
          return file;
        })
      );
      
      convertedFiles.value = files;
      loggerStore.success(`成功生成 ${files.length} 个待上传文件`);
      return files;
    } catch (error) {
      const errorMsg = `转换文件失败: ${error instanceof Error ? error.message : String(error)}`;
      loggerStore.error(errorMsg);
      console.error('转换文件失败:', error);
      throw new Error(errorMsg);
    } finally {
      converting.value = false;
    }
  }
  
  function clearConvertedFiles() {
    convertedFiles.value = [];
  }
  
  return {
    sourcePath,
    targetPath,
    scanResult,
    convertedFiles: readonly(convertedFiles),
    scanning: readonly(scanning),
    converting: readonly(converting),
    canConvert,
    canUpload,
    selectSourceFolder,
    selectTargetFolder,
    scanFiles,
    startConversion,
    clearConvertedFiles
  };
}