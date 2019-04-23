const originalResponse = new XMLHttpRequest().response

export function resetXhrPrototypeResponse(value) {
    Object.defineProperty(
        window.XMLHttpRequest.prototype,
        'response',
        { value: originalResponse },
    )
}

export function setXhrPrototypeResponse(value) {
    Object.defineProperty(
        window.XMLHttpRequest.prototype,
        'response',
        { value },
    )
}

export function setXhrPrototypeGetAllResponseHeaders(value) {
    Object.defineProperty(
        window.XMLHttpRequest.prototype,
        'getAllResponseHeaders',
        { value: () => value },
    )
}
