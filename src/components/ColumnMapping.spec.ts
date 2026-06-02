// @vitest-environment happy-dom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { defineComponent, nextTick, ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import ColumnMapping from './ColumnMapping.vue';
import type { ColumnMapping as ColumnMappingType } from '@/types';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
}));

describe('ColumnMapping', () => {
  it('does not recursively update when skipped mappings clear the model', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);

    const Parent = defineComponent({
      components: { ColumnMapping },
      setup() {
        const mappings = ref<ColumnMappingType[]>([
          { original: '原列名', mapped: '新列名' },
        ]);
        const updates = ref<ColumnMappingType[][]>([]);
        const saves = ref<ColumnMappingType[][]>([]);

        function handleUpdate(value: ColumnMappingType[]) {
          updates.value.push(value);
          mappings.value = value;
        }

        function handleSave(value: ColumnMappingType[]) {
          saves.value.push(value);
          mappings.value = value;
        }

        return { mappings, updates, saves, handleUpdate, handleSave };
      },
      template: `
        <ColumnMapping
          :model-value="mappings"
          @update:model-value="handleUpdate"
          @save="handleSave"
        />
      `,
    });

    const wrapper = mount(Parent, {
      global: {
        plugins: [pinia],
      },
    });

    const skipButton = wrapper
      .findAll('button')
      .find(button => button.text() === '跳过配置');

    expect(skipButton).toBeDefined();
    await skipButton!.trigger('click');
    await nextTick();
    await nextTick();

    expect(wrapper.vm.saves).toEqual([[]]);
    expect(wrapper.vm.updates).toHaveLength(0);
    expect(wrapper.vm.mappings).toEqual([]);
  });
});
