
let wasm;

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b);
                state.a = 0;

            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_20(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures__invoke1__hbeb08d93ab175161(arg0, arg1, addHeapObject(arg2));
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_23(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbe0a1ad32a1c5b7b(arg0, arg1);
}

let stack_pointer = 32;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}
function __wbg_adapter_26(arg0, arg1, arg2) {
    try {
        wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc382848871db0a84(arg0, arg1, addBorrowedObject(arg2));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

function handleError(f) {
    return function () {
        try {
            return f.apply(this, arguments);

        } catch (e) {
            wasm.__wbindgen_exn_store(addHeapObject(e));
        }
    };
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {

        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        var ret = false;
        return ret;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        var ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_activate_66268484bed1d269 = function(arg0, arg1) {
        mui.tabs.activate(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        var ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        var ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_new_59cb74e423758ede = function() {
        var ret = new Error();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_stack_558ba5917b466edd = function(arg0, arg1) {
        var ret = getObject(arg1).stack;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_error_4bb6c2a97407129a = function(arg0, arg1) {
        try {
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(arg0, arg1);
        }
    };
    imports.wbg.__wbg_Window_f826a1dec163bacb = function(arg0) {
        var ret = getObject(arg0).Window;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_WorkerGlobalScope_967d186155183d38 = function(arg0) {
        var ret = getObject(arg0).WorkerGlobalScope;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_instanceof_Window_fbe0320f34c4cd31 = function(arg0) {
        var ret = getObject(arg0) instanceof Window;
        return ret;
    };
    imports.wbg.__wbg_document_2b44f2a86e03665a = function(arg0) {
        var ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_location_df2a42f020b6b0fe = function(arg0) {
        var ret = getObject(arg0).location;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_alert_2bee8756b0c6ec99 = handleError(function(arg0, arg1, arg2) {
        getObject(arg0).alert(getStringFromWasm0(arg1, arg2));
    });
    imports.wbg.__wbg_getSelection_d88665146766882c = handleError(function(arg0) {
        var ret = getObject(arg0).getSelection();
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    });
    imports.wbg.__wbg_clearInterval_23bb8eb8b08345d0 = function(arg0, arg1) {
        getObject(arg0).clearInterval(arg1);
    };
    imports.wbg.__wbg_setInterval_006c702c8f19b4ce = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        return ret;
    });
    imports.wbg.__wbg_createElement_7cbe07ad3289abea = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_createElementNS_ae76308e06470c87 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
        var ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_createTextNode_bbff6f9f6e6b38bf = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getElementById_5bd6efc3d82494aa = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_querySelector_be2514e5de79507f = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    });
    imports.wbg.__wbg_deltaY_f2e82bfd030f24e4 = function(arg0) {
        var ret = getObject(arg0).deltaY;
        return ret;
    };
    imports.wbg.__wbg_setonmessage_fefcbee2824ce979 = function(arg0, arg1) {
        getObject(arg0).onmessage = getObject(arg1);
    };
    imports.wbg.__wbg_new_4ae6bf757712b3fd = handleError(function(arg0, arg1) {
        var ret = new Worker(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_newwithoptions_7960670defb5114d = handleError(function(arg0, arg1, arg2) {
        var ret = new Worker(getStringFromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_postMessage_58f317d71e0a2b3f = handleError(function(arg0, arg1) {
        getObject(arg0).postMessage(getObject(arg1));
    });
    imports.wbg.__wbg_createObjectURL_e46eb3ca9fdf20c5 = handleError(function(arg0, arg1) {
        var ret = URL.createObjectURL(getObject(arg1));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    });
    imports.wbg.__wbg_revokeObjectURL_a25352bc56969781 = handleError(function(arg0, arg1) {
        URL.revokeObjectURL(getStringFromWasm0(arg0, arg1));
    });
    imports.wbg.__wbg_readyState_bb1badda989804f6 = function(arg0) {
        var ret = getObject(arg0).readyState;
        return ret;
    };
    imports.wbg.__wbg_result_60d07c5337763fc4 = handleError(function(arg0) {
        var ret = getObject(arg0).result;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_new_de7aee5ca171b662 = handleError(function() {
        var ret = new FileReader();
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_abort_119fd8e51a24743b = function(arg0) {
        getObject(arg0).abort();
    };
    imports.wbg.__wbg_readAsArrayBuffer_3c2bca1bbe0a3889 = handleError(function(arg0, arg1) {
        getObject(arg0).readAsArrayBuffer(getObject(arg1));
    });
    imports.wbg.__wbg_instanceof_HtmlTextAreaElement_ceec64505a6bc087 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLTextAreaElement;
        return ret;
    };
    imports.wbg.__wbg_value_41a8d072fc94a8a4 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_setvalue_e3b8a9c5a4ad0114 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_instanceof_HtmlButtonElement_56d6508cc80478bf = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLButtonElement;
        return ret;
    };
    imports.wbg.__wbg_settype_ad8678a602cc664d = function(arg0, arg1, arg2) {
        getObject(arg0).type = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setdownload_9fdd241d1acb8083 = function(arg0, arg1, arg2) {
        getObject(arg0).download = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_href_389fc887040eaa21 = function(arg0, arg1) {
        var ret = getObject(arg1).href;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_sethref_a3c6793e49812099 = function(arg0, arg1, arg2) {
        getObject(arg0).href = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_data_b59dd3983cd7c614 = function(arg0) {
        var ret = getObject(arg0).data;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithstrsequenceandoptions_2c10c41f3bc24d7d = handleError(function(arg0, arg1) {
        var ret = new Blob(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_instanceof_HtmlInputElement_bd1ce15e756a8ae2 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLInputElement;
        return ret;
    };
    imports.wbg.__wbg_setchecked_1fe693236b4c24f3 = function(arg0, arg1) {
        getObject(arg0).checked = arg1 !== 0;
    };
    imports.wbg.__wbg_files_942fbf6943996106 = function(arg0) {
        var ret = getObject(arg0).files;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_settype_2628aa2562917c8e = function(arg0, arg1, arg2) {
        getObject(arg0).type = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_value_e27a74c3db49694f = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_setvalue_4f1af4fbd0b9942b = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_namespaceURI_5ab9ad8edd6b745e = function(arg0, arg1) {
        var ret = getObject(arg1).namespaceURI;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_setinnerHTML_8571d76e883c8748 = function(arg0, arg1, arg2) {
        getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_getAttribute_5d45357036c3ad3d = function(arg0, arg1, arg2, arg3) {
        var ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_removeAttribute_43b052e0560d223b = handleError(function(arg0, arg1, arg2) {
        getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
    });
    imports.wbg.__wbg_setAttribute_b638fce95071fff6 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    });
    imports.wbg.__wbg_click_5bcdaa3dfcd006fb = function(arg0) {
        getObject(arg0).click();
    };
    imports.wbg.__wbg_clearInterval_7d28b9e597277b67 = function(arg0, arg1) {
        getObject(arg0).clearInterval(arg1);
    };
    imports.wbg.__wbg_setInterval_e9c3336345e24578 = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        return ret;
    });
    imports.wbg.__wbg_preventDefault_4eb36ec8e5563ad6 = function(arg0) {
        getObject(arg0).preventDefault();
    };
    imports.wbg.__wbg_addEventListener_e8fdfac380f9ea25 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
    });
    imports.wbg.__wbg_removeEventListener_0007cd1449f82876 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
    });
    imports.wbg.__wbg_name_2b8f5aedebb9a225 = function(arg0, arg1) {
        var ret = getObject(arg1).name;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_get_971813fc163e7b0a = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_selectAllChildren_0342e558455c864e = handleError(function(arg0, arg1) {
        getObject(arg0).selectAllChildren(getObject(arg1));
    });
    imports.wbg.__wbg_origin_4a468d07705cfed2 = handleError(function(arg0, arg1) {
        var ret = getObject(arg1).origin;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    });
    imports.wbg.__wbg_nodeName_36e9b947eea43be6 = function(arg0, arg1) {
        var ret = getObject(arg1).nodeName;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_lastChild_5fc52139f307fbaa = function(arg0) {
        var ret = getObject(arg0).lastChild;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_setnodeValue_7404ea475784eb1c = function(arg0, arg1, arg2) {
        getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_appendChild_98dedaeac24501f2 = handleError(function(arg0, arg1) {
        var ret = getObject(arg0).appendChild(getObject(arg1));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_insertBefore_5886cc01dc0233e3 = handleError(function(arg0, arg1, arg2) {
        var ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_removeChild_51369e223cb8a779 = handleError(function(arg0, arg1) {
        var ret = getObject(arg0).removeChild(getObject(arg1));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_instanceof_HtmlSelectElement_9e453923d9cacda8 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLSelectElement;
        return ret;
    };
    imports.wbg.__wbg_value_b431982196f2362c = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_call_ab183a630df3a257 = handleError(function(arg0, arg1) {
        var ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_new_1abc33d4f9ba3e80 = function() {
        var ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_push_44968dcdf4cfbb43 = function(arg0, arg1) {
        var ret = getObject(arg0).push(getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_newnoargs_ab5e899738c0eff4 = function(arg0, arg1) {
        var ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_is_e8ad5aa6da4b8c83 = function(arg0, arg1) {
        var ret = Object.is(getObject(arg0), getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_new_dc5b27cfd2149b8f = function() {
        var ret = new Object();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_77eca7b42660e1bb = handleError(function() {
        var ret = self.self;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_window_51dac01569f1ba70 = handleError(function() {
        var ret = window.window;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_globalThis_34bac2d08ebb9b58 = handleError(function() {
        var ret = globalThis.globalThis;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_global_1c436164a66c9c22 = handleError(function() {
        var ret = global.global;
        return addHeapObject(ret);
    });
    imports.wbg.__wbg_buffer_bc64154385c04ac4 = function(arg0) {
        var ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_3c8748473807c7cf = function(arg0, arg1, arg2) {
        var ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_length_e9f6f145de2fede5 = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_new_22a33711cf65b661 = function(arg0) {
        var ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_b29de3f25280c6ec = function(arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_set_3afd31f38e771338 = handleError(function(arg0, arg1, arg2) {
        var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
        return ret;
    });
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        var ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        var ret = debugString(getObject(arg1));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function() {
        var ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper193 = function(arg0, arg1, arg2) {
        var ret = makeClosure(arg0, arg1, 30, __wbg_adapter_20);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper849 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 316, __wbg_adapter_23);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper850 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 316, __wbg_adapter_26);
        return addHeapObject(ret);
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    wasm.__wbindgen_start();
    return wasm;
}

export default init;

