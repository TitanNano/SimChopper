extends RefCounted

signal completed(results)
signal progress()

var _thread := Thread.new()
var _mutex := Mutex.new()
var _semaphore := Semaphore.new()
var _jobs := []
var runner: SceneTree
var executor: Callable
var _results := []

@warning_ignore("shadowed_variable")
func _init(runner: SceneTree, executor: Callable):
	self.runner = runner
	self.executor = executor

	# warning-ignore:return_value_discarded
	self._thread.start(self._thread_root)
	

func _thread_root(): 
	while true:
		prints(self.to_string(), "blocking thread for more jobs...")
		# warning-ignore:return_value_discarded
		self._semaphore.wait()
		prints(self.to_string(), "pumping thread...")
		
		self._mutex.lock()
		prints(self.to_string(), "thread has", self._jobs.size(), "jobs")
		var hasJobs = self._jobs.size() > 0
		self._mutex.unlock()
		
		prints(self.to_string(), "process jobs while we have some....")
		while hasJobs:
			self._mutex.lock()
			var job = self._jobs.pop_front()
			hasJobs = self._jobs.size() > 0
			self._mutex.unlock()
			
			var result = self.executor.call(job)
			
			self._results.append(result)
			self.call_deferred('emit_signal', "progress")

		if not hasJobs:
			prints(self.to_string(), "no jobs left, we are done...")
			return


func push(job_list):
	self._mutex.lock()
	self._jobs.append_array(job_list)
	self._mutex.unlock()
	
	# warning-ignore:return_value_discarded
	self._semaphore.post()


func drain():
	# warning-ignore:return_value_discarded
	self._semaphore.post()
	await self.runner.process_frame 	# make sure the method runs async
	
	while self._thread.is_alive():
		await self.runner.process_frame
	
	self._thread.wait_to_finish()
	
	var results := self._results
	self.completed.emit(results)
	self._results = []
