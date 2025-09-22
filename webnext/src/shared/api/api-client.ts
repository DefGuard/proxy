/** biome-ignore-all lint/suspicious/noExplicitAny: too generic to bother typing this strictly */
/** biome-ignore-all lint/complexity/noBannedTypes: needed */
import type { QueryKey } from '@tanstack/react-query';
import axios from 'axios';
import qs from 'qs';

const client = axios.create({
  baseURL: '/api/v1',
  headers: { 'Content-Type': 'application/json' },
  paramsSerializer: {
    serialize: (params) =>
      qs.stringify(params, {
        arrayFormat: 'repeat',
      }),
  },
});

type ApiResponse<T> = {
  data: T;
  status: number;
  invalidateKeys?: QueryKey[];
};

const RequestMethod = {
  Get: 'get',
  Post: 'post',
  Patch: 'patch',
  Delete: 'delete',
} as const;

type RequestMethodValue = (typeof RequestMethod)[keyof typeof RequestMethod];

type UrlLike<Path = void> = string | ((path: Path) => string);

type PathProp<Path> = [Path] extends [undefined] ? { path?: undefined } : { path: Path };

type GetDeleteProps<Params, Path> = PathProp<Path> &
  (Params extends void ? { params?: undefined } : { params: Params }) & {
    data?: never;
    abortSignal?: AbortSignal;
  };

type PostPatchProps<Body, Params, Path> = PathProp<Path> & {
  data: Body;
} & (Params extends never ? { params?: undefined } : { params: Params }) & {
    abortSignal?: AbortSignal;
  };

type RequestHandle<Fn> = {
  callbackFn: Fn;
  invalidateKeys?: QueryKey[];
};

type Cfg<M extends RequestMethodValue, Path> = {
  method: M;
  url: UrlLike<Path>;
  invalidateKeys?: QueryKey[];
};

function createRequest<Params = void, Res = unknown, Path = void>(
  cfg: Cfg<typeof RequestMethod.Get, Path> | Cfg<typeof RequestMethod.Delete, Path>,
): RequestHandle<(props: GetDeleteProps<Params, Path>) => Promise<ApiResponse<Res>>>;

function createRequest<Body = unknown, Res = unknown, Params = never, Path = void>(
  cfg: Cfg<typeof RequestMethod.Post, Path> | Cfg<typeof RequestMethod.Patch, Path>,
): RequestHandle<
  (props: PostPatchProps<Body, Params, Path>) => Promise<ApiResponse<Res>>
>;

function createRequest(
  cfg: Cfg<RequestMethodValue, any>,
): RequestHandle<(props: any) => Promise<ApiResponse<any>>> {
  const callbackFn = async (props: any = {}): Promise<ApiResponse<any>> => {
    const { abortSignal, path, ...rest } = props ?? {};
    const method = cfg.method;

    if ((method === 'get' || method === 'delete') && 'data' in rest) {
      throw new Error(`[${method.toUpperCase()}] must not include 'data'.`);
    }
    if ((method === 'post' || method === 'patch') && !('data' in rest)) {
      throw new Error(`[${method.toUpperCase()}] requires 'data'.`);
    }

    const finalUrl = typeof cfg.url === 'function' ? cfg.url(path) : cfg.url;

    const axiosRes = await client.request({
      url: finalUrl,
      method,
      ...(rest.params !== undefined ? { params: rest.params } : {}),
      ...(rest.data !== undefined ? { data: rest.data } : {}),
      signal: abortSignal,
    });

    return {
      data: axiosRes.data,
      status: axiosRes.status,
      invalidateKeys: cfg.invalidateKeys,
    };
  };

  return { callbackFn, invalidateKeys: cfg.invalidateKeys };
}

type HelperOpts = { invalidateKeys?: QueryKey[] };

export const get = <Params = void, Res = unknown, Path = void>(url: UrlLike<Path>) =>
  createRequest<Params, Res, Path>({ method: RequestMethod.Get, url });

export const del = <Params = void, Res = unknown, Path = void>(
  url: UrlLike<Path>,
  opts?: HelperOpts,
) =>
  createRequest<Params, Res, Path>({
    method: RequestMethod.Delete,
    url,
    invalidateKeys: opts?.invalidateKeys,
  });

export const post = <Body = unknown, Res = unknown, Params = never, Path = void>(
  url: UrlLike<Path>,
  opts?: HelperOpts,
) =>
  createRequest<Body, Res, Params, Path>({
    method: RequestMethod.Post,
    url,
    invalidateKeys: opts?.invalidateKeys,
  });

export const patch = <Body = unknown, Res = unknown, Params = never, Path = void>(
  url: UrlLike<Path>,
  opts?: HelperOpts,
) =>
  createRequest<Body, Res, Params, Path>({
    method: RequestMethod.Patch,
    url,
    invalidateKeys: opts?.invalidateKeys,
  });
