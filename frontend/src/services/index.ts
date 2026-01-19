import type { ApiService } from './api.interface';
import { api as realApi } from './api.real';
import { api as mockApi } from './api.mock';

const useMock = import.meta.env.VITE_USE_MOCK_API === 'true';

export const api: ApiService = useMock ? mockApi : realApi;
