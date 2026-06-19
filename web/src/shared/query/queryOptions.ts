import { queryOptions } from '@tanstack/react-query';
import { api } from '../api/api';
import { updateServiceApi } from '../api/update-service';

export const getAppInfoQueryOptions = queryOptions({
  queryFn: () => api.appInfo.callbackFn({}),
  queryKey: ['app-info'],
  select: (resp) => resp.data,
  staleTime: 60 * 1000,
  refetchOnWindowFocus: true,
  refetchOnMount: true,
  refetchOnReconnect: true,
});

export const getClientArtifactsQueryOptions = queryOptions({
  queryFn: updateServiceApi.getClientArtifacts,
  queryKey: ['update-service', 'artifacts'],
  staleTime: 180 * 1000,
  refetchOnWindowFocus: false,
  refetchOnMount: true,
  refetchOnReconnect: true,
});
