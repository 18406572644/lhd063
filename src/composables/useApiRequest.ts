import { ref, shallowRef } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { ApiError, type ApiErrorCode, type ApiResponse } from "@/types";

interface UseApiRequestOptions {
  maxRetries?: number;
  retryDelay?: number;
  showToast?: boolean;
  onSuccess?: (data: unknown) => void;
  onError?: (error: ApiError) => void;
}

const NETWORK_ERROR_MESSAGES: Record<string, boolean> = {
  Failed: true,
  NetworkError: true,
  network: true,
  timeout: true,
  aborted: true,
  disconnected: true,
  ERR_NETWORK: true,
};

function isNetworkError(error: unknown): boolean {
  if (error instanceof ApiError) {
    return error.code === "NETWORK_ERROR";
  }
  if (error instanceof Error) {
    const msg = error.message.toLowerCase();
    return Object.keys(NETWORK_ERROR_MESSAGES).some((key) =>
      msg.includes(key.toLowerCase())
    );
  }
  return false;
}

function categorizeError(error: unknown): ApiError {
  if (error instanceof ApiError) return error;

  if (isNetworkError(error)) {
    return new ApiError(
      { code: "NETWORK_ERROR", message: "网络连接失败，请检查" },
      error
    );
  }

  if (error instanceof Error) {
    const msg = error.message.toLowerCase();

    if (msg.includes("not found") || msg.includes("404")) {
      return new ApiError(
        { code: "NOT_FOUND", message: "请求的资源不存在" },
        error
      );
    }
    if (
      msg.includes("validation") ||
      msg.includes("invalid") ||
      msg.includes("required")
    ) {
      return new ApiError(
        { code: "VALIDATION_ERROR", message: error.message },
        error
      );
    }
    if (msg.includes("unauthorized") || msg.includes("401")) {
      return new ApiError(
        { code: "UNAUTHORIZED", message: "未授权，请重新登录" },
        error
      );
    }
    if (msg.includes("forbidden") || msg.includes("403")) {
      return new ApiError(
        { code: "FORBIDDEN", message: "没有操作权限" },
        error
      );
    }
    if (msg.includes("conflict") || msg.includes("409") || msg.includes("duplicate")) {
      return new ApiError(
        { code: "CONFLICT", message: "数据冲突，请刷新后重试" },
        error
      );
    }

    return new ApiError(
      { code: "INTERNAL_ERROR", message: error.message },
      error
    );
  }

  return new ApiError(
    { code: "UNKNOWN_ERROR", message: "操作失败，请重试" },
    undefined
  );
}

function getDefaultMessage(code: ApiErrorCode): string {
  const messages: Record<ApiErrorCode, string> = {
    NETWORK_ERROR: "网络连接失败，请检查",
    NOT_FOUND: "请求的资源不存在",
    VALIDATION_ERROR: "数据验证失败",
    UNAUTHORIZED: "未授权，请重新登录",
    FORBIDDEN: "没有操作权限",
    CONFLICT: "数据冲突，请刷新后重试",
    INTERNAL_ERROR: "操作失败，请重试",
    UNKNOWN_ERROR: "操作失败，请重试",
  };
  return messages[code];
}

function reportError(error: ApiError, context?: string) {
  console.error(`[ApiError] ${context || "Unhandled"}`, {
    code: error.code,
    message: error.message,
    detail: error.detail,
    field: error.field,
    fields: error.fields,
  });
}

export function useApiRequest<T = unknown>(options: UseApiRequestOptions = {}) {
  const {
    maxRetries = 0,
    retryDelay = 1000,
    showToast = true,
    onSuccess,
    onError,
  } = options;

  const loading = ref(false);
  const error = shallowRef<ApiError | null>(null);
  const data = shallowRef<T | null>(null);

  const router = useRouter();

  function handleError(apiError: ApiError) {
    if (apiError.code === "NOT_FOUND") {
      router.push("/404");
      return;
    }

    if (apiError.code === "VALIDATION_ERROR") {
      if (showToast) {
        ElMessage.warning({
          message: apiError.message || "数据验证失败",
          duration: 4000,
          showClose: true,
        });
      }
      return;
    }

    if (apiError.code === "NETWORK_ERROR") {
      if (showToast) {
        ElMessage.error({
          message: "网络连接失败，请检查",
          duration: 5000,
          showClose: true,
        });
      }
      reportError(apiError);
      return;
    }

    if (showToast) {
      ElMessage.error({
        message: apiError.message || getDefaultMessage(apiError.code),
        duration: 5000,
        showClose: true,
      });
    }
    reportError(apiError);
  }

  async function execute<R>(
    requestFn: () => Promise<ApiResponse<R>>
  ): Promise<ApiResponse<R>> {
    loading.value = true;
    error.value = null;

    let lastError: ApiError | null = null;
    let attempts = maxRetries + 1;

    for (let attempt = 1; attempt <= attempts; attempt++) {
      try {
        const response = await requestFn();

        if (!response.success && response.error) {
          const apiError = new ApiError(response.error);
          error.value = apiError;
          handleError(apiError);
          onError?.(apiError);
          loading.value = false;
          return response;
        }

        data.value = response.data as unknown as T;
        onSuccess?.(response.data);
        loading.value = false;
        return response;
      } catch (err: unknown) {
        lastError = categorizeError(err);
        error.value = lastError;

        const isRetryable =
          lastError.code === "NETWORK_ERROR" ||
          lastError.code === "INTERNAL_ERROR";

        if (isRetryable && attempt < attempts) {
          await new Promise((resolve) => setTimeout(resolve, retryDelay));
          continue;
        }

        handleError(lastError);
        onError?.(lastError);
        loading.value = false;

        return {
          success: false,
          data: null as unknown as R,
          error: {
            code: lastError.code,
            message: lastError.message,
            field: lastError.field,
            fields: lastError.fields,
          },
        };
      }
    }

    loading.value = false;
    const fallbackError =
      lastError ||
      new ApiError({ code: "UNKNOWN_ERROR", message: "操作失败，请重试" });
    error.value = fallbackError;
    handleError(fallbackError);
    onError?.(fallbackError);

    return {
      success: false,
      data: null as unknown as R,
      error: {
        code: fallbackError.code,
        message: fallbackError.message,
      },
    } as ApiResponse<R>;
  }

  function clearError() {
    error.value = null;
  }

  return {
    loading,
    error,
    data,
    execute,
    clearError,
  };
}

export { categorizeError, reportError, getDefaultMessage };
