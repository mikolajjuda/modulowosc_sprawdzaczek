package main

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
)

func judge(image string, submission map[string]interface{}) map[string]interface{} {
	json_bytes, err := json.Marshal(submission)
	if err != nil {
		panic(err)
	}

	ctx := context.Background()
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		panic(err)
	}
	defer cli.Close()

	pids_limit := int64(100)

	resp, err := cli.ContainerCreate(ctx,
		&container.Config{
			Image:     image,
			OpenStdin: true,
			StdinOnce: true,
			Tty:       false,
		},
		&container.HostConfig{
			Resources: container.Resources{
				Memory:     1024 * 1024 * 1024,
				MemorySwap: 2 * 1024 * 1024 * 1024,
				NanoCPUs:   500000000,
				PidsLimit:  &pids_limit,
			},
		}, nil, nil, "")
	if err != nil {
		panic(err)
	}

	if err := cli.ContainerStart(ctx, resp.ID, types.ContainerStartOptions{}); err != nil {
		panic(err)
	}

	conn, err := cli.ContainerAttach(ctx, resp.ID,
		types.ContainerAttachOptions{Stream: true, Stdout: true, Stderr: true, Stdin: true})
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	_, err = conn.Conn.Write(json_bytes)
	if err != nil {
		panic(err)
	}
	_, err = conn.Conn.Write([]byte("\n"))
	if err != nil {
		panic(err)
	}
	fmt.Println("Sent submission to judge")

	var container_out bytes.Buffer
	container_out_buf := bufio.NewWriter(&container_out)
	go stdcopy.StdCopy(container_out_buf, os.Stderr, conn.Reader)

	statusCh, errCh := cli.ContainerWait(ctx, resp.ID, container.WaitConditionNotRunning)
	select {
	case err := <-errCh:
		if err != nil {
			panic(err)
		}
	case <-statusCh:
	}
	fmt.Println("Judge finished")
	container_out_buf.Flush()

	inspection, err := cli.ContainerInspect(ctx, resp.ID)
	if err != nil {
		panic(err)
	}

	if err := cli.ContainerRemove(ctx, resp.ID, types.ContainerRemoveOptions{}); err != nil {
		panic(err)
	}

	if inspection.State.ExitCode != 0 {
		fmt.Println(container_out.String())
		fmt.Println("Judge exited with non-zero exit code")
	} else {
		var result map[string]interface{}
		if err := json.Unmarshal(container_out.Bytes(), &result); err != nil {
			panic(err)
		}
		return result
	}
	return nil
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: judge_manager <config>")
		return
	}
	submission_json_path := os.Args[1]
	submission_json_file, err := os.Open(submission_json_path)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	defer submission_json_file.Close()
	submission_json_bytes, err := io.ReadAll(submission_json_file)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	var dat map[string]interface{}
	if err := json.Unmarshal(submission_json_bytes, &dat); err != nil {
		fmt.Println("Error:", err)
		return
	}
	if dat["task_type"] == "simple_diff" && dat["lang"] == "cpp" {
		result := judge("simple_diff_cpp", dat)
		fmt.Println(result)
	} else if dat["task_type"] == "custom_portable" {
		result := judge(dat["image"].(string), dat)
		fmt.Println(result)
	} else {
		fmt.Println("Error: unsupported task type or language")
	}
}
