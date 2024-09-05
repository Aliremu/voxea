package main

import (
    . "example.com/api"
)

type PluginApiImpl struct {
}

func (i PluginApiImpl) Enable() int32 {
    return 5
}

func (i PluginApiImpl) Disable() int32 {
    return 10
}

func (i PluginApiImpl) ProcessSignal(ptr uint64) {
}

func init() {
    example := PluginApiImpl{}
    SetExportsSdkComponentPluginApi(example)
}

func main() {}