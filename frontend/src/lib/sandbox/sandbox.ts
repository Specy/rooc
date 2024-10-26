import {SANDBOX_RUNTIME_DATA} from "$lib/sandbox/sandboxRuntimeData";
import std from '../../../static/std/bundle.js?raw'
export type ConsoleOutput = {
    type: 'log' | 'error' | 'warn' | 'info';
    args: unknown[];
};


export async function runSandboxedCode(code: string, global: Record<string, unknown> = {}): Promise<{
    result: any;
    consoleOutput: ConsoleOutput[]
}> {
    return new Promise((resolve) => {
        const iframe = document.createElement('iframe');
        iframe.sandbox.add("allow-same-origin")
        iframe.sandbox.add("allow-scripts")
        iframe.style.display = 'none';
        document.body.appendChild(iframe);

        const consoleOutput: ConsoleOutput[] = [];

        const iframeWindow = iframe.contentWindow;
        if (!iframeWindow) {
            throw new Error('Failed to create iframe');
        }

        for (const [key, value] of Object.entries(global)) {
            iframeWindow[key] = value
        }

        // Override console methods
        (['log', 'error', 'warn', 'info']).forEach((method) => {
            //@ts-expect-error
            iframeWindow.console[method as keyof Console] = (...args: any[]) => {
                console[method](...args);
                consoleOutput.push({type: method as 'log' | 'error' | 'warn' | 'info', args});
            };
        });

        // Inject code to signal completion
        const wrappedCode = `
        ${std}
        //because it's a namespace
        Rooc = Rooc.Rooc
      // Disable dangerous APIs
      const dangerousApis = [
        'localStorage',
        'sessionStorage',
        'indexedDB',
        'webkitIndexedDB',
        'mozIndexedDB',
        'msIndexedDB',
        'fetch',
        'XMLHttpRequest',
        'WebSocket',
        'EventSource',
        'webkitStorageInfo',
        'crypto',
        'caches',
        'navigation',
        'navigator',
        'screen',
        'history',
        'document',
        'self',
        'opener',
        'window',
        'top',
        'frames',
        'parent',
        'frameElement',
        'external',
        'status',
        'name',
        'close',
        'open',
        'alert',
        'confirm',
        'prompt',
        'print',
        'blur',
        'focus',       
      ];
      let ignoreWarning = true
      const postMessage = window.parent.postMessage;
      function disableApi(obj, api) {
         obj[api] = undefined
         delete obj[api]
         try{
            Object.defineProperty(obj, api, {
              get: () => {
                if(!ignoreWarning) console.warn(\`Access to \${api} is not allowed in this sandbox.\`);
                return undefined;
              },
              set: () => {
                if(!ignoreWarning) console.warn(\`Setting \${api} is not allowed in this sandbox.\`);
              },
            });
        }catch(e){}
      }

      // Disable APIs on both window and globalThis
      [window, globalThis].forEach(obj => {
        dangerousApis.forEach(api => {
            disableApi(obj, api);
        });

      });

        ignoreWarning = false;
       (async function() {
          try {
            ${SANDBOX_RUNTIME_DATA}
            
            ${code}
          } catch (error) {
            console.error(error);
          } finally {
            postMessage({ type: 'CODE_EXECUTION_COMPLETE', result: (typeof result !== 'undefined') ? result : undefined }, '*');
          }
       })()

    `;

        // Listen for the completion message
        window.addEventListener('message', function onMessage(event) {
            if (event.data.type === 'CODE_EXECUTION_COMPLETE') {
                window.removeEventListener('message', onMessage);
                document.body.removeChild(iframe);
                resolve({result: event.data.result, consoleOutput});
            }
        });

        // Execute the code
        const script = iframeWindow.document.createElement('script');
        script.textContent = wrappedCode;
        iframeWindow.document.body.appendChild(script);
    });
}