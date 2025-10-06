package com.api.javinhaApi.service;

import org.springframework.stereotype.Service;

import com.api.javinhaApi.dto.UserDto;
import com.api.javinhaApi.model.User;
import com.api.javinhaApi.repository.UserRepository;

@Service
public class GreetingService {

    private final UserRepository userRepository;

    public GreetingService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    public String greet(String name) {
        if (name == null || name.isBlank()) return "Hello, World!";
        return "Hello, " + name + "!";
    }

    public UserDto saveUser(UserDto dto) {
        User u = new User();
        u.setName(dto.getName());
        u.setEmail(dto.getEmail());
        User saved = userRepository.save(u);
        return new UserDto(saved.getId(), saved.getName(), saved.getEmail());
    }
}
