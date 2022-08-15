package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"syscall"

	//"github.com/schollz/progressbar/v3"
	"github.com/vbauerster/mpb/v7"
)

func DownloadFile(url string, dest string, name string, progressName string, p *mpb.Progress) {
	fileName := name //path.Base(url)

	filePath := fmt.Sprintf("%s/%s.tmp", dest, fileName)
	out, err := os.Create(filePath)

	if err != nil {
		fmt.Println(filePath)
		panic(err)
	}

	defer out.Close()

	resp, err := http.Get(url)

	if err != nil {
		panic(err)
	}

	defer resp.Body.Close()

	/* bar := progressbar.DefaultBytes(
		resp.ContentLength,
		fmt.Sprintf("Downloading %s", progressName),
	) */

	_, err = io.Copy(io.MultiWriter(out /* bar */), resp.Body)

	if err != nil {
		panic(err)
	}

	os.Rename(filePath, strings.Replace(filePath, ".tmp", "", -1))
}

func cleanup() {
	files, err := filepath.Glob("*.img*")
	if err != nil {
		fmt.Println(err)
	}

	for _, file := range files {
		if err := os.Remove(file); err != nil {
			fmt.Println(err)
		}
	}
}

type Template struct {
	VMID int
	Name string
	Link string `json:"-"`
}

func main() {
	c := make(chan os.Signal)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-c
		cleanup()
		os.Exit(1)
	}()

	reader := bufio.NewReader(os.Stdin)
	fmt.Println("Storage location to import VMs (e.g. local, local-lvm)")
	fmt.Print("Location: ")
	location, _ := reader.ReadString('\n')

	images := []Template{
		{
			VMID: 1000,
			Name: "Ubuntu 22.04",
			Link: "https://cloud-images.ubuntu.com/focal/current/focal-server-cloudimg-amd64.img",
		},
		{
			VMID: 1001,
			Name: "Ubuntu 20.04",
			Link: "https://cloud-images.ubuntu.com/focal/current/focal-server-cloudimg-amd64.img",
		},
		{
			VMID: 1002,
			Name: "Ubuntu 18.04",
			Link: "https://cloud-images.ubuntu.com/bionic/current/bionic-server-cloudimg-amd64.img",
		},
	}

	var wg sync.WaitGroup
	p := mpb.New(mpb.WithWaitGroup(&wg))

	for _, image := range images {
		fileName := strconv.Itoa(image.VMID) + ".img"
		wg.Add(1)

		go func(image Template) {
			if _, err := os.Stat(fileName); err != nil {
				DownloadFile(image.Link, "./", fileName, image.Name, p)
			} else {
				fmt.Printf("Image '%s' already exists!\n", image.Name)
			}

			wg.Done()
		}(image)
	}

	p.Wait()

	fmt.Printf("Importing VMs to %s", location)

	for _, image := range images {
		fmt.Printf("Importing %s (vmid: %d)\n", image.Name, image.VMID)

		err := exec.Command("bash", "-c", fmt.Sprintf("qm create %d --memory 2048 --net0 virtio,bridge=vmbr0", image.VMID)).Run()
		if err != nil {
			fmt.Printf("Failed to import %s (vmid: %d)\n", image.Name, image.VMID)
			continue
		}

		err = exec.Command("bash", "-c", fmt.Sprintf("qm importdisk %d %s %s", image.VMID, strconv.Itoa(image.VMID)+".img", location)).Run()
		if err != nil {
			log.Fatal(err)
		}
	}

	fmt.Println("Cleaning up...")
	cleanup()
}
