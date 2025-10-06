package com.api.javinhaApi.controller;

import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import com.api.javinhaApi.dto.UserDto;
import com.api.javinhaApi.service.GreetingService;

@RestController
@RequestMapping("/api/v1")
public class GreetingController {

    private final GreetingService greetingService;

    public GreetingController(GreetingService greetingService) {
        this.greetingService = greetingService;
    }

    @GetMapping("/greet")
    public ResponseEntity<String> greet(@RequestParam(required = false) String name) {
        return ResponseEntity.ok(greetingService.greet(name));
    }

    @PostMapping("/users")
    public ResponseEntity<UserDto> createUser(@RequestBody UserDto dto) {
        UserDto saved = greetingService.saveUser(dto);
        return ResponseEntity.ok(saved);
    }
}
