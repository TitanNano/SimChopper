extends RefCounted

const ThreadWorker := preload("ThreadWorker.gd")

signal completed(results)
signal progress(status: Dictionary)

var pool: Array[ThreadWorker]= []
var _completed_workers := 0
var _completed_jobs := 0
var _total_job_count := 0
var _mutex = Mutex.new()
var _results = []

func _collect_thread(thread_results: Array):
	self._mutex.lock()
	self._completed_workers += 1
	self._results.append_array(thread_results)

	if self._completed_workers < self.pool.size():
		return
		
	var results = self._results
	self._results = []
	# find out why the results are so huge. 1.2 GB is not reasonable
	self.completed.emit(results)
	self._mutex.unlock()


func _report_porgress():
	self._completed_jobs += 1
	self.progress.emit({ "complete": self._completed_jobs, "total": self._total_job_count })


func _init(runner: SceneTree,executor: Callable):
	for _i in range(0, OS.get_processor_count()):
		var worker := ThreadWorker.new(runner, executor)
		worker.completed.connect(self._collect_thread)
		worker.progress.connect(self._report_porgress)

		self.pool.append(worker)

func fill(jobs: Array):
	var cursor := 0
	var size := int(ceilf(jobs.size() / float(self.pool.size())))
	
	self._total_job_count += jobs.size()
	
	for worker in self.pool:
		var chunk := jobs.slice(cursor, cursor + size - 1)
		
		cursor += size
		worker.push(chunk)

	for worker in self.pool:
		worker.drain()
