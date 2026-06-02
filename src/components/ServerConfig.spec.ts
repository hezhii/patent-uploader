// @vitest-environment happy-dom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { defineComponent, nextTick, ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import ServerConfig from './ServerConfig.vue';
import type { ServerConfig as ServerConfigType } from '@/types';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
}));

describe('ServerConfig', () => {
  it('emits one model update when editing the server URL', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);

    const Parent = defineComponent({
      components: { ServerConfig },
      setup() {
        const config = ref<ServerConfigType>({
          serverUrl: '',
          username: '',
          password: '',
        });
        const updates = ref<ServerConfigType[]>([]);

        function handleUpdate(value: ServerConfigType) {
          updates.value.push(value);
          config.value = value;
        }

        return { config, updates, handleUpdate };
      },
      template: `
        <ServerConfig
          :model-value="config"
          @update:model-value="handleUpdate"
        />
      `,
    });

    const wrapper = mount(Parent, {
      global: {
        plugins: [pinia],
      },
    });

    await wrapper.find('input[type="url"]').setValue('https://api.example.com');
    await nextTick();
    await nextTick();

    expect(wrapper.vm.updates).toHaveLength(1);
    expect(wrapper.vm.config.serverUrl).toBe('https://api.example.com');
  });
});
